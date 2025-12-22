use super::constants::*;
use super::error::ParsError;
use super::transaction::*;
use super::utils::{read_byte, remove_quotes};
use chrono::DateTime;
use std::collections::HashMap;
use std::io::{Read, Write};

const HEADER_VALUES: [&str; CNT_VALUES] = [
    TX_ID,
    TX_TYPE,
    FROM_USER_ID,
    TO_USER_ID,
    AMOUNT,
    TIMESTAMP,
    STATUS,
    DESCRIPTION,
];

enum Token {
    Value(String),
    EndOfLine(String),
    EndOfStream(Option<String>),
}

#[derive(Clone, Copy)]
enum ParserState {
    WaitStartRecord,
    WaitStartValue,
    WaitEndRegular,
    WaitEndString,
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
        let mut buf = Vec::new();
        loop {
            let byte = match read_byte(&mut self.stream) {
                Ok(val) => val,
                Err(e) => match e {
                    ParsError::EndOfStream => {
                        let res = std::str::from_utf8(&buf)?.trim().to_string();
                        if res.is_empty() {
                            return Ok(Token::EndOfStream(None));
                        } else {
                            return Ok(Token::EndOfStream(Some(res)));
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

                    if byte == '"' as u8 {
                        buf.push(byte);
                        self.state = ParserState::WaitEndString;
                        continue;
                    }

                    buf.push(byte);
                    self.state = ParserState::WaitEndRegular;
                }
                ParserState::WaitStartValue => {
                    if byte == ' ' as u8 {
                        continue;
                    }

                    if byte == '"' as u8 {
                        buf.push(byte);
                        self.state = ParserState::WaitEndString;
                        continue;
                    }
                    buf.push(byte);
                    self.state = ParserState::WaitEndRegular;
                }
                ParserState::WaitEndRegular => {
                    if byte == ',' as u8 {
                        let val_text = std::str::from_utf8(&buf)?.trim();
                        self.state = ParserState::WaitStartValue;
                        return Ok(Token::Value(val_text.to_owned()));
                    }

                    if byte == '\n' as u8 {
                        let val_text = std::str::from_utf8(&buf)?.trim();
                        self.state = ParserState::WaitStartRecord;
                        return Ok(Token::EndOfLine(val_text.to_owned()));
                    }
                    buf.push(byte);
                }

                ParserState::WaitEndString => {
                    if byte == '\\' as u8 {
                        self.state = ParserState::WaitEscaped;
                        continue;
                    }
                    if byte == '"' as u8 {
                        buf.push(byte);
                        self.state = ParserState::WaitEndRegular;
                        continue;
                    }
                    buf.push(byte);
                }
                ParserState::WaitEscaped => {
                    buf.push(byte);
                    self.state = ParserState::WaitEndString;
                    continue;
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct CsvTxRecord {
    fields: Vec<String>,
}

impl CsvTxRecord {
    fn serialize<Out: Write>(&self, out: &mut Out) -> Result<(), ParsError> {
        let mut res = String::new();
        for (idx, val) in self.fields.iter().enumerate() {
            if idx > 0 {
                res.push(',');
            }
            res.push_str(val);
        }
        res.push('\n');
        out.write_all(res.as_bytes())?;
        Ok(())
    }

    fn to_transaction(&self, header: &HashMap<String, usize>) -> Result<Transaction, ParsError> {
        if self.fields.len() != header.len() {
            return Err(ParsError::WrongFormat(
                "Количество полей не соответствует заголовку".to_owned(),
            ));
        }

        let tx_id = self.fields[header[TX_ID]].parse::<u64>()?;
        let tx_type = self.fields[header[TX_TYPE]].as_str();
        let tx_type = match tx_type {
            DEPOSIT => TxType::Deposit,
            TRANSFER => TxType::Transfer,
            WITHDRAWAL => TxType::Withdrawal,
            _ => {
                return Err(ParsError::WrongFormat(format!(
                    "Неверный формат TX_TYPE: {tx_type}"
                )));
            }
        };

        let from_user_id = self.fields[header[FROM_USER_ID]].parse::<u64>()?;
        let to_user_id = self.fields[header[TO_USER_ID]].parse::<u64>()?;
        let amount = self.fields[header[AMOUNT]].parse::<i64>()?;
        let timestamp = self.fields[header[TIMESTAMP]].parse::<u64>()?;
        let timestamp = if let Some(val) = DateTime::from_timestamp_millis(timestamp as i64) {
            val
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Wrong timestamp: {}",
                timestamp
            )));
        };

        let status = self.fields[header[STATUS]].as_str();
        let status = match status {
            SUCCESS => TxStatus::Success,
            FAILURE => TxStatus::Failure,
            PENDING => TxStatus::Pending,
            _ => {
                return Err(ParsError::WrongFormat(format!(
                    "Неверный формат STATUS: {status}"
                )));
            }
        };

        let description = self.fields[header[DESCRIPTION]].as_str();

        if !(description.starts_with('"') && description.ends_with('"')) {
            return Err(ParsError::WrongFormat(format!(
                "Wrong description: {}",
                description
            )));
        }

        Ok(Transaction {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description: remove_quotes(&description),
        })
    }

    fn from_transaction(tx: &Transaction, header: &HashMap<String, usize>) -> Self {
        let mut fields = vec![String::new(); CNT_VALUES];
        fields[header[TX_ID]] = tx.tx_id.to_string();
        fields[header[TX_TYPE]] = match tx.tx_type {
            TxType::Deposit => DEPOSIT.to_owned(),
            TxType::Transfer => TRANSFER.to_owned(),
            TxType::Withdrawal => WITHDRAWAL.to_owned(),
        };
        fields[header[FROM_USER_ID]] = tx.from_user_id.to_string();
        fields[header[TO_USER_ID]] = tx.to_user_id.to_string();
        fields[header[AMOUNT]] = tx.amount.to_string();
        let timestamp = tx.timestamp.timestamp_millis() as u64;
        fields[header[TIMESTAMP]] = timestamp.to_string();
        fields[header[STATUS]] = match tx.status {
            TxStatus::Success => SUCCESS.to_owned(),
            TxStatus::Failure => FAILURE.to_owned(),
            TxStatus::Pending => PENDING.to_owned(),
        };
        fields[header[DESCRIPTION]] = format!("\"{}\"", tx.description);
        Self { fields }
    }
}

pub struct CsvTxReader<In: Read> {
    parser: Parser<In>,
    header: Option<HashMap<String, usize>>,
}

impl<In: Read> CsvTxReader<In> {
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self {
            parser: Parser::new(stream),
            header: None,
        })
    }

    fn read_values(&mut self) -> Result<Vec<String>, ParsError> {
        let mut res = Vec::new();
        loop {
            match self.parser.get_next_token()? {
                Token::Value(val) => res.push(val),
                Token::EndOfLine(val) => {
                    res.push(val);
                    return Ok(res);
                }
                Token::EndOfStream(val) => {
                    if let Some(reminder) = val {
                        res.push(reminder);
                    }
                    return Ok(res);
                }
            }
        }
    }

    fn read_header(&mut self) -> Result<(), ParsError> {
        let header = self.read_values()?;
        if header != HEADER_VALUES {
            return Err(ParsError::WrongFormat(format!(
                "Неверный заголовок: {:?}",
                header
            )));
        }

        let res: HashMap<String, usize> = header
            .into_iter()
            .enumerate()
            .map(|(idx, name)| (name, idx))
            .collect();
        self.header = Some(res);
        Ok(())
    }

    pub fn read_transaction(&mut self) -> Result<Option<Transaction>, ParsError> {
        if self.header.is_none() {
            self.read_header()?;
        }
        let fields = self.read_values()?;
        if fields.is_empty() {
            return Ok(None);
        }
        let csv_record = CsvTxRecord { fields };

        if let Some(header) = self.header.as_ref() {
            Ok(Some(csv_record.to_transaction(header)?))
        } else {
            return Err(ParsError::WrongFormat("Отсутствует заголовок".to_owned()));
        }
    }
}

pub struct CsvTxWriter<Out: Write> {
    stream: Out,
    header: Option<HashMap<String, usize>>,
}

impl<Out: Write> CsvTxWriter<Out> {
    pub fn new(stream: Out) -> Result<Self, ParsError> {
        Ok(Self {
            stream,
            header: None,
        })
    }

    pub fn write_header(&mut self) -> Result<(), ParsError> {
        let mut header_str = String::new();
        for (idx, field) in HEADER_VALUES.into_iter().enumerate() {
            if idx > 0 {
                header_str.push(',');
            }
            header_str.push_str(field);
        }
        header_str.push('\n');
        self.stream.write_all(header_str.as_bytes())?;

        let header: HashMap<String, usize> = HEADER_VALUES
            .into_iter()
            .enumerate()
            .map(|(idx, name)| (name.to_string(), idx))
            .collect();
        self.header = Some(header);
        Ok(())
    }

    pub fn write_transaction(&mut self, data: &Transaction) -> Result<(), ParsError> {
        if self.header.is_none() {
            self.write_header()?;
        }

        if let Some(header) = self.header.as_ref() {
            let record = CsvTxRecord::from_transaction(&data, header);
            record.serialize(&mut self.stream)?;
        } else {
            return Err(ParsError::WrongFormat("Не записан заголовок".to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const EXPECTED_CSV: &str = "1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n";
    const EXPECTED_CSV_MULT: &str = r#"
        TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"

        1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,"Record number 2"
    "#;

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

    fn csv_record_for_test() -> CsvTxRecord {
        CsvTxRecord {
            fields: vec![
                "1000000000000000".to_owned(),
                "DEPOSIT".to_owned(),
                "0".to_owned(),
                "9223372036854775807".to_owned(),
                "100".to_owned(),
                "1633036860000".to_owned(),
                "FAILURE".to_owned(),
                "\"Record number 1\"".to_owned(),
            ],
        }
    }

    fn get_header() -> HashMap<String, usize> {
        let res: HashMap<String, usize> = HEADER_VALUES
            .into_iter()
            .enumerate()
            .map(|(idx, name)| (name.to_string(), idx))
            .collect();
        res
    }

    #[test]
    fn test_csv_from_transaction() {
        let tx = tx1_for_test();
        let expected = csv_record_for_test();
        let header = get_header();
        let record = CsvTxRecord::from_transaction(&tx, &header);

        assert_eq!(record, expected);
    }

    #[test]
    fn test_csv_to_transaction() {
        let csv_record = csv_record_for_test();
        let expected = tx1_for_test();
        let header = get_header();
        let tx = csv_record.to_transaction(&header).unwrap();

        assert_eq!(tx, expected);
    }

    #[test]
    fn test_serialize_csv_record() {
        let record = csv_record_for_test();
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        record.serialize(&mut cursor).unwrap();

        assert_eq!(std::str::from_utf8(cursor.get_ref()).unwrap(), EXPECTED_CSV);
    }

    #[test]
    fn test_csv_reader() {
        let stream = Cursor::new(EXPECTED_CSV_MULT.as_bytes());
        let mut csv_reader = CsvTxReader::new(stream).unwrap();

        let mut fin_info = Vec::new();
        while let Some(tx) = csv_reader.read_transaction().unwrap() {
            fin_info.push(tx);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], tx1_for_test());
        assert_eq!(fin_info[1], tx2_for_test());
    }

    #[test]
    fn test_csv_writer() {
        let buf = Vec::new();
        let stream = Cursor::new(buf);
        let mut csv_writer = CsvTxWriter::new(stream).unwrap();

        csv_writer.write_transaction(&tx1_for_test()).unwrap();
        csv_writer.write_transaction(&tx2_for_test()).unwrap();

        let buf = csv_writer.stream.get_ref();
        let stream = Cursor::new(buf);
        let mut csv_reader = CsvTxReader::new(stream).unwrap();
        let mut fin_info = Vec::new();
        while let Some(tx) = csv_reader.read_transaction().unwrap() {
            fin_info.push(tx);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], tx1_for_test());
        assert_eq!(fin_info[1], tx2_for_test());
    }
}
