use super::constants::*;
use super::error::ParsError;
use super::finance_data::*;
use super::utils::remove_quotes;
use chrono::DateTime;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

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

fn get_next_not_empty<In: Read>(stream: &mut BufReader<In>) -> Result<Vec<String>, ParsError> {
    let mut values = String::new();
    loop {
        match stream.read_line(&mut values) {
            Ok(0) => return Err(ParsError::EndOfStream),
            Err(e) => return Err(e.into()),
            Ok(_) => {
                values = values.trim().to_owned();
                if !values.is_empty() {
                    break;
                }
            }
        }
    }

    let values: Vec<String> = values
        .split(',')
        .map(|value| value.trim().to_owned())
        .collect();

    Ok(values)
}

#[derive(Eq, PartialEq, Debug)]
struct CsvFinanceRecord {
    fields: Vec<String>,
}

impl CsvFinanceRecord {
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

    fn deserialize<In: Read>(input: &mut BufReader<In>) -> Result<Self, ParsError> {
        Ok(Self {
            fields: get_next_not_empty(input)?,
        })
    }

    fn to_fin_data(&self, header: &HashMap<String, usize>) -> Result<FinanceData, ParsError> {
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

        Ok(FinanceData {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description: remove_quotes(description),
        })
    }

    fn from_fin_data(fin_data: &FinanceData, header: &HashMap<String, usize>) -> Self {
        let mut fields = vec![String::new(); CNT_VALUES];
        fields[header[TX_ID]] = fin_data.tx_id.to_string();
        fields[header[TX_TYPE]] = match fin_data.tx_type {
            TxType::Deposit => DEPOSIT.to_owned(),
            TxType::Transfer => TRANSFER.to_owned(),
            TxType::Withdrawal => WITHDRAWAL.to_owned(),
        };
        fields[header[FROM_USER_ID]] = fin_data.from_user_id.to_string();
        fields[header[TO_USER_ID]] = fin_data.to_user_id.to_string();
        fields[header[AMOUNT]] = fin_data.amount.to_string();
        let timestamp = fin_data.timestamp.timestamp_millis() as u64;
        fields[header[TIMESTAMP]] = timestamp.to_string();
        fields[header[STATUS]] = match fin_data.status {
            TxStatus::Success => SUCCESS.to_owned(),
            TxStatus::Failure => FAILURE.to_owned(),
            TxStatus::Pending => PENDING.to_owned(),
        };
        fields[header[DESCRIPTION]] = format!("\"{}\"", fin_data.description);
        Self { fields }
    }
}

pub struct CsvReader<In: Read> {
    stream: BufReader<In>,
    header: Option<HashMap<String, usize>>,
}

impl<In: Read> CsvReader<In> {
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self {
            stream: BufReader::new(stream),
            header: None,
        })
    }

    fn read_header(&mut self) -> Result<(), ParsError> {
        let header = get_next_not_empty(&mut self.stream)?;
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

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        if self.header.is_none() {
            self.read_header()?;
        }
        let record = match CsvFinanceRecord::deserialize(&mut self.stream) {
            Ok(val) => val,
            Err(e) => {
                if let ParsError::EndOfStream = e {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if let Some(header) = self.header.as_ref() {
            Ok(Some(record.to_fin_data(header)?))
        } else {
            return Err(ParsError::WrongFormat("Отсутствует заголовок".to_owned()));
        }
    }
}

pub struct CsvWriter<Out: Write> {
    stream: Out,
    header: Option<HashMap<String, usize>>,
}

impl<Out: Write> CsvWriter<Out> {
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

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError> {
        if self.header.is_none() {
            self.write_header()?;
        }

        if let Some(header) = self.header.as_ref() {
            let record = CsvFinanceRecord::from_fin_data(&data, header);
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

    fn fin_data1_for_test() -> FinanceData {
        FinanceData {
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

    fn fin_data2_for_test() -> FinanceData {
        FinanceData {
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

    fn csv_record_for_test() -> CsvFinanceRecord {
        CsvFinanceRecord {
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
    fn test_csv_from_finance_data() {
        let fin_data = fin_data1_for_test();
        let expected = csv_record_for_test();
        let header = get_header();
        let record = CsvFinanceRecord::from_fin_data(&fin_data, &header);

        assert_eq!(record, expected);
    }

    #[test]
    fn test_csv_to_finance_data() {
        let csv_record = csv_record_for_test();
        let expected = fin_data1_for_test();
        let header = get_header();
        let fin_data = csv_record.to_fin_data(&header).unwrap();

        assert_eq!(fin_data, expected);
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
    fn test_deserialize_csv_record() {
        let expected = csv_record_for_test();
        let mut buf = BufReader::new(Cursor::new(EXPECTED_CSV.as_bytes()));
        let record = CsvFinanceRecord::deserialize(&mut buf).unwrap();

        assert_eq!(record, expected);
    }

    #[test]
    fn test_csv_reader() {
        let stream = Cursor::new(EXPECTED_CSV_MULT.as_bytes());
        let mut csv_reader = CsvReader::new(stream).unwrap();

        let mut fin_info = Vec::new();
        while let Some(fin_data) = csv_reader.read_fin_data().unwrap() {
            fin_info.push(fin_data);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], fin_data1_for_test());
        assert_eq!(fin_info[1], fin_data2_for_test());
    }

    #[test]
    fn test_csv_writer() {
        let buf = Vec::new();
        let stream = Cursor::new(buf);
        let mut csv_writer = CsvWriter::new(stream).unwrap();

        csv_writer.write_fin_data(&fin_data1_for_test()).unwrap();
        csv_writer.write_fin_data(&fin_data2_for_test()).unwrap();

        let buf = csv_writer.stream.get_ref();
        let stream = Cursor::new(buf);
        let mut csv_reader = CsvReader::new(stream).unwrap();
        let mut fin_info = Vec::new();
        while let Some(fin_data) = csv_reader.read_fin_data().unwrap() {
            fin_info.push(fin_data);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], fin_data1_for_test());
        assert_eq!(fin_info[1], fin_data2_for_test());
    }
}
