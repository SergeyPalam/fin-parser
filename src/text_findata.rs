use super::parser::*;
use super::error::ParsError;
use super::constants::*;
use std::io::{Read, Write, BufRead, BufReader};
use chrono::DateTime;
use std::collections::HashMap;

struct TextFinanceRecord {
    fields: HashMap<String, String>,
}

impl TextFinanceRecord {
    fn serialize<Out: Write>(&self, out: &mut Out) -> Result<(), ParsError> {
        for (k, v) in self.fields.iter() {
            let line = format!("{k}: {v}\n");
            out.write(line.as_bytes())?;
        }
        out.write(b"\n")?;
        Ok(())
    }

    fn deserialize<In: Read>(stream: &mut BufReader<In>) -> Result<Self, ParsError> {
        let mut line = String::new();
        loop{
            stream.read_line(&mut line)?;
            let trimmed = line.trim();
            if !trimmed.starts_with('#') && !trimmed.is_empty(){
                break;
            }
        }
        let mut fields = HashMap::new();

        loop {
            let (key, val) =
            if let Some((k, v)) = line.split_once(':'){
                (k.trim().to_owned(), v.trim().to_owned())
            }else{
                return Err(ParsError::WrongFormat(format!("Неверный входной формат: {line}")));
            };
            fields.insert(key, val);
            stream.read_line(&mut line)?;
            if line.trim().is_empty() {
                break;
            }
        }
        Ok(Self{
            fields
        })
    }
}

pub struct TextReader<In: Read>{
    stream: BufReader<In>,
}

impl <In: Read> TextReader<In>{
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self{
            stream: BufReader::new(stream),
        })
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        let record = match TextFinanceRecord::deserialize(&mut self.stream){
            Ok(val) => {
                val
            }
            Err(e) => {
                if let ParsError::EndOfStream = e {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if record.fields.len() != CNT_VALUES {
            return Err(ParsError::WrongFormat(format!("Неверрный формат записи: {:?}", record.fields)));
        }

        let tx_id =
        if let Some(val) = record.fields.get(TX_ID){
            val.parse::<u64>()?
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {TX_ID}")));
        };

        let tx_type =
        if let Some(val) = record.fields.get(TX_TYPE){
            match val.as_str() {
                DEPOSIT => TxType::Deposit,
                TRANSFER => TxType::Transfer,
                WITHDRAWAL => TxType::Withdrawal,
                _ => {
                    return Err(ParsError::WrongFormat(format!("Неверный тип транзакции: {val}")));
                }
            }
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {TX_ID}")));
        };

        let from_user_id =
        if let Some(val) = record.fields.get(FROM_USER_ID){
            val.parse::<u64>()?
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {FROM_USER_ID}")));
        };

        let to_user_id =
        if let Some(val) = record.fields.get(TO_USER_ID){
            val.parse::<u64>()?
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {TO_USER_ID}")));
        };

        let amount =
        if let Some(val) = record.fields.get(AMOUNT){
            val.parse::<i64>()?
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {AMOUNT}")));
        };

        let timestamp =
        if let Some(val) = record.fields.get(TIMESTAMP){
            let millis = val.parse::<u64>()?;
            if let Some(date_time) = DateTime::from_timestamp_millis(millis as i64){
                date_time
            }else{
                return Err(ParsError::WrongFormat(format!("Неверный формат времени: {millis}")));
            }
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {TIMESTAMP}")));
        };

        let status =
        if let Some(val) = record.fields.get(STATUS){
            match val.as_str() {
                SUCCESS => TxStatus::Success,
                FAILURE => TxStatus::Failure,
                PENDING => TxStatus::Pending,
                _ => {
                    return Err(ParsError::WrongFormat(format!("Неверный статус транзакции: {val}")));
                }
            }
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {STATUS}")));
        };

        let description =
        if let Some(val) = record.fields.get(DESCRIPTION){
            if let Some(val) = val.strip_prefix("\"").map(|val| val.strip_suffix("\"")).flatten(){
                val.to_owned()
            }else{
                return Err(ParsError::WrongFormat(format!("Неверный формат description: {val}")));
            }
        }else{
            return Err(ParsError::WrongFormat(format!("Отсутствует запись: {DESCRIPTION}")));
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

pub struct TextWriter<Out: Write>{
    stream: Out,
}

impl<Out: Write> TextWriter<Out>{
    pub fn new(stream: Out) -> Result<Self, ParsError>{
        Ok(Self{
            stream,
        })
    }

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError>{
        let mut fields = HashMap::new();
        fields.insert(TX_ID.to_owned(), data.tx_id.to_string());
        let tx_type = match data.tx_type {
            TxType::Deposit => DEPOSIT,
            TxType::Transfer => TRANSFER,
            TxType::Withdrawal => WITHDRAWAL,
        };
        fields.insert(TX_TYPE.to_owned(), tx_type.to_owned());
        fields.insert(FROM_USER_ID.to_owned(), data.from_user_id.to_string());
        fields.insert(TO_USER_ID.to_owned(), data.to_user_id.to_string());
        fields.insert(AMOUNT.to_owned(), data.amount.to_string());
        let timestamp = data.timestamp.timestamp_millis() as u64;
        fields.insert(TIMESTAMP.to_owned(), timestamp.to_string());
        let status = match data.status {
            TxStatus::Success => SUCCESS,
            TxStatus::Failure => FAILURE,
            TxStatus::Pending => PENDING,
        };
        fields.insert(STATUS.to_owned(), status.to_string());
        let description = format!("\"{}\"", data.description);
        fields.insert(DESCRIPTION.to_owned(), description.to_string());
        todo!();
    }
}
