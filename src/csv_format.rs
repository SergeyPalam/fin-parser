use super::constants::*;
use super::error::ParsError;
use super::finance_data::*;
use chrono::DateTime;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

const HEADER_VALUES: [(&str, usize); CNT_VALUES] = [
    (TX_ID, 0),
    (TX_TYPE, 1),
    (FROM_USER_ID, 2),
    (TO_USER_ID, 3),
    (AMOUNT, 4),
    (TIMESTAMP, 5),
    (STATUS, 6),
    (DESCRIPTION, 7),
];

fn get_next_not_empty<In: Read>(stream: &mut BufReader<In>) -> Result<Vec<String>, ParsError> {
    let mut values = String::new();
    while values.is_empty() {
        stream.read_line(&mut values)?;
    }

    let values: Vec<String> = values
        .split(',')
        .map(|value| value.trim().to_owned())
        .collect();

    Ok(values)
}

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
        out.write_all(res.as_bytes())?;
        Ok(())
    }

    fn deserialize<In: Read>(input: &mut BufReader<In>) -> Result<Self, ParsError> {
        Ok(Self {
            fields: get_next_not_empty(input)?,
        })
    }

    fn to_fin_data(&self, header: &HashMap<&'static str, usize>) -> Result<FinanceData, ParsError> {
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

        let description = if let Some(val) = description
            .strip_prefix("\"")
            .map(|val| val.strip_suffix("\""))
            .flatten()
        {
            val.to_owned()
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Неверный формат description: {description}"
            )));
        };

        Ok(FinanceData {
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

    fn from_fin_data(fin_data: &FinanceData, header: &HashMap<&'static str, usize>) -> Self {
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
    header: HashMap<&'static str, usize>,
}

impl<In: Read> CsvReader<In> {
    pub fn new(stream: In) -> Result<Self, ParsError> {
        let mut stream = BufReader::new(stream);
        let header: Vec<(String, usize)> = get_next_not_empty(&mut stream)?
            .into_iter()
            .enumerate()
            .map(|(idx, name)| (name, idx))
            .collect();

        if header.len() != HEADER_VALUES.len() {
            return Err(ParsError::WrongFormat("Неверный заголовок".to_owned()));
        }

        for (lhs, rhs) in HEADER_VALUES.iter().zip(header.iter()) {
            if !(lhs.0 == rhs.0 && lhs.1 == rhs.1) {
                return Err(ParsError::WrongFormat("Неверный заголовок".to_owned()));
            }
        }

        Ok(Self {
            stream,
            header: HashMap::from_iter(HEADER_VALUES.into_iter()),
        })
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
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

        Ok(Some(record.to_fin_data(&self.header)?))
    }
}

pub struct CsvWriter<Out: Write> {
    stream: Out,
    header: HashMap<&'static str, usize>,
}

impl<Out: Write> CsvWriter<Out> {
    pub fn new(mut stream: Out) -> Result<Self, ParsError> {
        let mut header = String::new();
        for (field, idx) in HEADER_VALUES {
            if idx > 0 {
                header.push(',');
            }
            header.push_str(field);
        }
        header.push('\n');
        stream.write_all(header.as_bytes())?;
        Ok(Self {
            stream,
            header: HashMap::from_iter(HEADER_VALUES.into_iter()),
        })
    }

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError> {
        let record = CsvFinanceRecord::from_fin_data(&data, &self.header);
        record.serialize(&mut self.stream)?;
        Ok(())
    }
}
