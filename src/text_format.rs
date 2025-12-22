use super::constants::*;
use super::error::ParsError;
use super::transaction::*;
use super::utils::{read_byte, remove_quotes};
use chrono::DateTime;
use std::collections::HashMap;
use std::io::{Read, Write};

enum Token {
    KeyValue((String, String)),
    SplitRecords,
    EndOfStream(Option<(String, String)>),
}

#[derive(Clone, Copy)]
enum PrevParserState {
    WaitStartRecord,
    WaitStartKey,
}

#[derive(Clone, Copy)]
enum ParserState {
    WaitStartRecord,
    WaitStartKey,
    WaitEndKey,
    WaitStartValue,
    WaitEndRegular,
    WaitEndString,
    WaitEndComment(PrevParserState),
    WaitEscaped,
}

struct Parser<In: Read> {
    state: ParserState,
    stream: In,
}

impl<In: Read> Parser<In> {
    fn new(stream: In) -> Self {
        Self {
            state: ParserState::WaitStartRecord,
            stream,
        }
    }

    fn get_next_token(&mut self) -> Result<Token, ParsError> {
        let mut key_buf = Vec::new();
        let mut val_buf = Vec::new();
        loop {
            let byte = match read_byte(&mut self.stream) {
                Ok(val) => val,
                Err(e) => match e {
                    ParsError::EndOfStream => {
                        let key_text = std::str::from_utf8(&key_buf)?.trim().to_string();
                        let val_text = std::str::from_utf8(&val_buf)?.trim().to_string();
                        if !(key_text.is_empty() && val_text.is_empty()) {
                            return Ok(Token::EndOfStream(Some((key_text, val_text))));
                        } else {
                            return Ok(Token::EndOfStream(None));
                        }
                    }
                    _ => {
                        return Err(e);
                    }
                },
            };
            match self.state {
                ParserState::WaitStartRecord => {
                    if byte == ' ' as u8 || byte == '\n' as u8 {
                        continue;
                    }

                    if byte == '#' as u8 {
                        self.state = ParserState::WaitEndComment(PrevParserState::WaitStartRecord);
                        continue;
                    }

                    key_buf.push(byte);
                    self.state = ParserState::WaitEndKey;
                }
                ParserState::WaitStartKey => {
                    if byte == ' ' as u8 {
                        continue;
                    }

                    if byte == '#' as u8 {
                        self.state = ParserState::WaitEndComment(PrevParserState::WaitStartKey);
                        continue;
                    }

                    if byte == '\n' as u8 {
                        self.state = ParserState::WaitStartRecord;
                        return Ok(Token::SplitRecords);
                    }

                    key_buf.push(byte);
                    self.state = ParserState::WaitEndKey;
                }

                ParserState::WaitEndKey => {
                    if byte == ':' as u8 {
                        self.state = ParserState::WaitStartValue;
                        continue;
                    }
                    key_buf.push(byte);
                }

                ParserState::WaitStartValue => {
                    if byte == ' ' as u8 {
                        continue;
                    }
                    val_buf.push(byte);

                    if byte == '"' as u8 {
                        self.state = ParserState::WaitEndString;
                        continue;
                    }
                    self.state = ParserState::WaitEndRegular;
                }

                ParserState::WaitEndRegular => {
                    if byte == '\n' as u8 {
                        let key_text = std::str::from_utf8(&key_buf)?.trim().to_string();
                        let val_text = std::str::from_utf8(&val_buf)?.trim().to_string();
                        self.state = ParserState::WaitStartKey;
                        return Ok(Token::KeyValue((key_text, val_text)));
                    }
                    val_buf.push(byte);
                }

                ParserState::WaitEndString => {
                    if byte == '\\' as u8 {
                        self.state = ParserState::WaitEscaped;
                        continue;
                    }
                    val_buf.push(byte);
                    if byte == '"' as u8 {
                        self.state = ParserState::WaitEndRegular;
                        continue;
                    }
                }
                ParserState::WaitEscaped => {
                    val_buf.push(byte);
                    self.state = ParserState::WaitEndString;
                    continue;
                }
                ParserState::WaitEndComment(prev_state) => {
                    if byte == '\n' as u8 {
                        match prev_state {
                            PrevParserState::WaitStartKey => {
                                self.state = ParserState::WaitStartKey;
                            }
                            PrevParserState::WaitStartRecord => {
                                self.state = ParserState::WaitStartRecord;
                            }
                        }
                    }
                }
            }
        }
    }
}

struct TextTxRecord {
    fields: HashMap<String, String>,
}

impl TextTxRecord {
    fn serialize<Out: Write>(&self, out: &mut Out) -> Result<(), ParsError> {
        for (k, v) in self.fields.iter() {
            let line = format!("{k}: {v}\n");
            out.write(line.as_bytes())?;
        }
        out.write(b"\n")?;
        Ok(())
    }

    fn to_transaction(&self) -> Result<Transaction, ParsError> {
        if self.fields.len() != CNT_VALUES {
            return Err(ParsError::WrongFormat(format!(
                "Неверрный формат записи: {:?}",
                self.fields
            )));
        }

        let tx_id = if let Some(val) = self.fields.get(TX_ID) {
            val.parse::<u64>()?
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {TX_ID}"
            )));
        };

        let tx_type = if let Some(val) = self.fields.get(TX_TYPE) {
            match val.as_str() {
                DEPOSIT => TxType::Deposit,
                TRANSFER => TxType::Transfer,
                WITHDRAWAL => TxType::Withdrawal,
                _ => {
                    return Err(ParsError::WrongFormat(format!(
                        "Неверный тип транзакции: {val}"
                    )));
                }
            }
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {TX_ID}"
            )));
        };

        let from_user_id = if let Some(val) = self.fields.get(FROM_USER_ID) {
            val.parse::<u64>()?
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {FROM_USER_ID}"
            )));
        };

        let to_user_id = if let Some(val) = self.fields.get(TO_USER_ID) {
            val.parse::<u64>()?
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {TO_USER_ID}"
            )));
        };

        let amount = if let Some(val) = self.fields.get(AMOUNT) {
            val.parse::<i64>()?
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {AMOUNT}"
            )));
        };

        let timestamp = if let Some(val) = self.fields.get(TIMESTAMP) {
            let millis = val.parse::<u64>()?;
            if let Some(date_time) = DateTime::from_timestamp_millis(millis as i64) {
                date_time
            } else {
                return Err(ParsError::WrongFormat(format!(
                    "Неверный формат времени: {millis}"
                )));
            }
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {TIMESTAMP}"
            )));
        };

        let status = if let Some(val) = self.fields.get(STATUS) {
            match val.as_str() {
                SUCCESS => TxStatus::Success,
                FAILURE => TxStatus::Failure,
                PENDING => TxStatus::Pending,
                _ => {
                    return Err(ParsError::WrongFormat(format!(
                        "Неверный статус транзакции: {val}"
                    )));
                }
            }
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {STATUS}"
            )));
        };

        let description = if let Some(val) = self.fields.get(DESCRIPTION) {
            if !(val.starts_with('"') && val.ends_with('"')) {
                return Err(ParsError::WrongFormat(format!(
                    "Wrong description: {}",
                    val
                )));
            }
            remove_quotes(&val)
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Отсутствует запись: {DESCRIPTION}"
            )));
        };

        Ok(Transaction {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        })
    }

    fn from_transaction(tx: &Transaction) -> Self {
        let mut fields = HashMap::new();
        fields.insert(TX_ID.to_owned(), tx.tx_id.to_string());
        let tx_type = match tx.tx_type {
            TxType::Deposit => DEPOSIT,
            TxType::Transfer => TRANSFER,
            TxType::Withdrawal => WITHDRAWAL,
        };
        fields.insert(TX_TYPE.to_owned(), tx_type.to_owned());
        fields.insert(FROM_USER_ID.to_owned(), tx.from_user_id.to_string());
        fields.insert(TO_USER_ID.to_owned(), tx.to_user_id.to_string());
        fields.insert(AMOUNT.to_owned(), tx.amount.to_string());
        let timestamp = tx.timestamp.timestamp_millis() as u64;
        fields.insert(TIMESTAMP.to_owned(), timestamp.to_string());
        let status = match tx.status {
            TxStatus::Success => SUCCESS,
            TxStatus::Failure => FAILURE,
            TxStatus::Pending => PENDING,
        };
        fields.insert(STATUS.to_owned(), status.to_string());
        let description = format!("\"{}\"", tx.description);
        fields.insert(DESCRIPTION.to_owned(), description.to_string());

        Self { fields }
    }
}

pub struct TextTxReader<In: Read> {
    parser: Parser<In>,
}

impl<In: Read> TextTxReader<In> {
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self {
            parser: Parser::new(stream),
        })
    }

    pub fn read_transaction(&mut self) -> Result<Option<Transaction>, ParsError> {
        let mut fields = HashMap::new();
        loop {
            let token = self.parser.get_next_token()?;
            match token {
                Token::KeyValue((k, v)) => {
                    fields.insert(k, v);
                }
                Token::SplitRecords => {
                    break;
                }
                Token::EndOfStream(reminder) => {
                    if let Some((k, v)) = reminder {
                        fields.insert(k, v);
                    }
                    break;
                }
            }
        }

        if fields.is_empty() {
            return Ok(None);
        }

        let text_record = TextTxRecord { fields };

        Ok(Some(text_record.to_transaction()?))
    }
}

pub struct TextTxWriter<Out: Write> {
    stream: Out,
}

impl<Out: Write> TextTxWriter<Out> {
    pub fn new(stream: Out) -> Result<Self, ParsError> {
        Ok(Self { stream })
    }

    pub fn write_transaction(&mut self, data: &Transaction) -> Result<(), ParsError> {
        let record = TextTxRecord::from_transaction(&data);
        record.serialize(&mut self.stream)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const EXPECTED_TEXT_MULT: &str = r#"
        # Record 1 (DEPOSIT)
        TX_TYPE: DEPOSIT
        TO_USER_ID: 9223372036854775807
        FROM_USER_ID: 0
        TIMESTAMP: 1633036860000
        DESCRIPTION: "Record number 1"
        TX_ID: 1000000000000000
        AMOUNT: 100
        STATUS: FAILURE

        # Record 2 (TRANSFER)
        DESCRIPTION: "Record number 2"
        TIMESTAMP: 1633036920000
        STATUS: PENDING
        AMOUNT: 200
        TX_ID: 1000000000000001
        TX_TYPE: TRANSFER
        FROM_USER_ID: 9223372036854775807
        TO_USER_ID: 9223372036854775807
    "#;

    fn eq_hash_maps(lhs: &HashMap<String, String>, rhs: &HashMap<String, String>) -> bool {
        if lhs.len() != rhs.len() {
            return false;
        }

        let res = lhs.iter().all(|lhs_item| {
            if let Some(rhs_val) = rhs.get(lhs_item.0) {
                if lhs_item.1 == rhs_val { true } else { false }
            } else {
                false
            }
        });

        res
    }

    fn tx1_for_test() -> Transaction {
        Transaction {
            tx_id: 1000000000000000,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 9223372036854775807,
            amount: 100,
            timestamp: DateTime::from_timestamp_millis(1633036860000 as i64).unwrap(),
            status: TxStatus::Failure,
            description: "Record number 1".to_owned(),
        }
    }

    fn tx2_for_test() -> Transaction {
        Transaction {
            tx_id: 1000000000000001,
            tx_type: TxType::Transfer,
            from_user_id: 9223372036854775807,
            to_user_id: 9223372036854775807,
            amount: 200,
            timestamp: DateTime::from_timestamp_millis(1633036920000 as i64).unwrap(),
            status: TxStatus::Pending,
            description: "Record number 2".to_owned(),
        }
    }

    fn text_record_for_test() -> TextTxRecord {
        let mut fields = HashMap::new();
        fields.insert(TX_ID.to_owned(), "1000000000000000".to_owned());
        fields.insert(TX_TYPE.to_owned(), "DEPOSIT".to_owned());
        fields.insert(FROM_USER_ID.to_owned(), "0".to_owned());
        fields.insert(TO_USER_ID.to_owned(), "9223372036854775807".to_owned());
        fields.insert(AMOUNT.to_owned(), "100".to_owned());
        fields.insert(TIMESTAMP.to_owned(), "1633036860000".to_owned());
        fields.insert(STATUS.to_owned(), "FAILURE".to_owned());
        fields.insert(DESCRIPTION.to_owned(), "\"Record number 1\"".to_owned());
        TextTxRecord { fields }
    }

    #[test]
    fn test_text_to_transaction() {
        let text_record = text_record_for_test();
        let expected = tx1_for_test();
        let tx = text_record.to_transaction().unwrap();

        assert_eq!(tx, expected);
    }

    #[test]
    fn test_text_from_transaction() {
        let tx = tx1_for_test();
        let expected = text_record_for_test();
        let record = TextTxRecord::from_transaction(&tx);

        assert!(eq_hash_maps(&record.fields, &expected.fields));
    }

    #[test]
    fn test_text_reader() {
        let stream = Cursor::new(EXPECTED_TEXT_MULT.as_bytes());
        let mut csv_reader = TextTxReader::new(stream).unwrap();

        let mut fin_info = Vec::new();
        while let Some(tx) = csv_reader.read_transaction().unwrap() {
            fin_info.push(tx);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], tx1_for_test());
        assert_eq!(fin_info[1], tx2_for_test());
    }

    #[test]
    fn test_text_writer() {
        let buf = Vec::new();
        let stream = Cursor::new(buf);
        let mut csv_writer = TextTxWriter::new(stream).unwrap();

        csv_writer.write_transaction(&tx1_for_test()).unwrap();
        csv_writer.write_transaction(&tx2_for_test()).unwrap();

        let buf = csv_writer.stream.get_ref();
        let stream = Cursor::new(buf);
        let mut text_reader = TextTxReader::new(stream).unwrap();
        let mut fin_info = Vec::new();
        while let Some(tx) = text_reader.read_transaction().unwrap() {
            fin_info.push(tx);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], tx1_for_test());
        assert_eq!(fin_info[1], tx2_for_test());
    }
}
