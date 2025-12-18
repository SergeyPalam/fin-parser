use super::parser::*;
use super::error::ParsError;
use std::{io::{BufRead, BufReader, Read, Write}};
use chrono::DateTime;
use std::collections::HashMap;

const TX_ID: &str = "TX_ID";
const TX_TYPE: &str = "TX_TYPE";
const FROM_USER_ID: &str = "FROM_USER_ID";
const TO_USER_ID: &str = "TO_USER_ID";
const AMOUNT: &str = "AMOUNT";
const TIMESTAMP: &str = "TIMESTAMP";
const STATUS: &str = "STATUS";
const DESCRIPTION: &str = "DESCRIPTION";

const DEPOSIT: &str = "DEPOSIT";
const TRANSFER: &str = "TRANSFER";
const WITHDRAWAL: &str = "WITHDRAWAL";

const SUCCESS: &str = "SUCCESS";
const FAILURE: &str = "FAILURE";
const PENDING: &str = "PENDING";

const CNT_VALUES: usize = 8;
const HEADER_VALUES: [(&str, usize); CNT_VALUES] = 
        [(TX_ID, 0),
        (TX_TYPE, 1),
        (FROM_USER_ID, 2),
        (TO_USER_ID, 3),
        (AMOUNT, 4),
        (TIMESTAMP, 5),
        (STATUS, 6),
        (DESCRIPTION, 7)];

fn get_next_not_empty<In: Read>(stream: &mut BufReader<In>) -> Result<Vec<String>, ParsError> {
    let mut values = String::new();
    while values.is_empty() {
        stream.read_line(&mut values)?;
    }

    let values: Vec<String> = values.split(',').map(|value| {
        value.trim().to_owned()
    }).collect();

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
        Ok(Self{
            fields: get_next_not_empty(input)?,
        })
    }
}

pub struct CsvReader<In: Read>{
    stream: BufReader<In>,
    header: HashMap<&'static str, usize>,
}

impl <In: Read> CsvReader<In>{
    pub fn new(stream: In) -> Result<Self, ParsError> {
        let mut stream = BufReader::new(stream);
        let header: Vec<(String, usize)> = get_next_not_empty(&mut stream)?.into_iter().
            enumerate().
            map(|(idx, name)|{
                (name, idx)
            }).collect();

        if header.len() != HEADER_VALUES.len() {
            return Err(ParsError::WrongFormat("Неверный заголовок".to_owned()))
        }

        for (lhs, rhs) in HEADER_VALUES.iter().zip(header.iter()) {
            if !(lhs.0 == rhs.0 && lhs.1 == rhs.1){
                return Err(ParsError::WrongFormat("Неверный заголовок".to_owned()))
            }
        }

        Ok(Self {
            stream,
            header: HashMap::from_iter(HEADER_VALUES.into_iter())
        })
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        let record = match CsvFinanceRecord::deserialize(&mut self.stream){
            Ok(val) => val,
            Err(e) => {
                if let ParsError::EndOfStream = e {
                    return Ok(None);
                }else{
                    return Err(e);
                }
            }
        };

        if record.fields.len() != self.header.len() {
            return Err(ParsError::WrongFormat("Количество полей не соответствует заголовку".to_owned()));
        }

        let tx_id = record.fields[self.header[TX_ID]].parse::<u64>()?;
        let tx_type = record.fields[self.header[TX_TYPE]].as_str();
        let tx_type =
        match tx_type {
            DEPOSIT => TxType::Deposit,
            TRANSFER => TxType::Transfer,
            WITHDRAWAL => TxType::Withdrawal,
            _ => {
                return Err(ParsError::WrongFormat(format!("Неверный формат TX_TYPE: {tx_type}")));
            }
        };

        let from_user_id = record.fields[self.header[FROM_USER_ID]].parse::<u64>()?;
        let to_user_id = record.fields[self.header[TO_USER_ID]].parse::<u64>()?;
        let amount = record.fields[self.header[AMOUNT]].parse::<i64>()?;
        let timestamp = record.fields[self.header[TIMESTAMP]].parse::<u64>()?;
        let timestamp = if let Some(val) = DateTime::from_timestamp_millis(timestamp as i64){
            val
        }else{
            return Err(ParsError::WrongFormat(format!("Wrong timestamp: {}", timestamp)));
        };

        let status = record.fields[self.header[STATUS]].as_str();
        let status =
        match status {
            SUCCESS => TxStatus::Success,
            FAILURE => TxStatus::Failure,
            PENDING => TxStatus::Pending,
            _ => {
                return Err(ParsError::WrongFormat(format!("Неверный формат STATUS: {status}")));
            }
        };

        let description = record.fields[self.header[DESCRIPTION]].as_str();

        let description =
        if let Some(val) = description.strip_prefix("\"").map(|val| val.strip_suffix("\"")).flatten(){
            val.to_owned()
        }else{
            return Err(ParsError::WrongFormat(format!("Неверный формат description: {description}")));
        };

        Ok(Some(FinanceData{
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        }))
    }
}

pub struct CsvWriter<Out: Write>{
    stream: Out,
    header: HashMap<&'static str, usize>,
}

impl<Out: Write> CsvWriter<Out>{
    pub fn new(mut stream: Out) -> Result<Self, ParsError>{
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
            header: HashMap::from_iter(HEADER_VALUES.into_iter())
        })
    }

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError>{
        let mut fields = vec![String::new(); CNT_VALUES];
        fields[self.header[TX_ID]] = data.tx_id.to_string();
        fields[self.header[TX_TYPE]] =
        match data.tx_type {
            TxType::Deposit => DEPOSIT.to_owned(),
            TxType::Transfer => TRANSFER.to_owned(),
            TxType::Withdrawal => WITHDRAWAL.to_owned(),
        };
        fields[self.header[FROM_USER_ID]] = data.from_user_id.to_string();
        fields[self.header[TO_USER_ID]] = data.to_user_id.to_string();
        fields[self.header[AMOUNT]] = data.amount.to_string();
        let timestamp = data.timestamp.timestamp_millis() as u64;
        fields[self.header[TIMESTAMP]] = timestamp.to_string();
        fields[self.header[STATUS]] =
        match data.status {
            TxStatus::Success => SUCCESS.to_owned(),
            TxStatus::Failure => FAILURE.to_owned(),
            TxStatus::Pending => PENDING.to_owned(),
        };
        fields[self.header[DESCRIPTION]] = format!("\"{}\"", data.description);
        let record = CsvFinanceRecord {
            fields,
        };
        record.serialize(&mut self.stream)?;
        Ok(())
    }
}
