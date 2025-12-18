use super::parser::*;
use super::error::ParsError;
use std::io::{Read, Write, BufReader};
use chrono::DateTime;

const MAGIC: u32 = 0x5950424E;

fn read_u8<T: Read>(stream: &mut T) -> Result<u8, ParsError> {
    let mut buf = [0u8; std::mem::size_of::<u8>()];
    stream.read_exact(&mut buf)?;
    let res = u8::from_be_bytes(buf);
    Ok(res)
}

fn read_u32<T: Read>(stream: &mut T) -> Result<u32, ParsError> {
    let mut buf = [0u8; std::mem::size_of::<u32>()];
    stream.read_exact(&mut buf)?;
    let res = u32::from_be_bytes(buf);
    Ok(res)
}

fn read_u64<T: Read>(stream: &mut T) -> Result<u64, ParsError> {
    let mut buf = [0u8; std::mem::size_of::<u64>()];
    stream.read_exact(&mut buf)?;
    let res = u64::from_be_bytes(buf);
    Ok(res)
}

fn read_i64<T: Read>(stream: &mut T) -> Result<i64, ParsError> {
    let mut buf = [0u8; std::mem::size_of::<i64>()];
    stream.read_exact(&mut buf)?;
    let res = i64::from_be_bytes(buf);
    Ok(res)
}

struct BinFinanceRecord {
    magic: u32,
    record_size: u32,
    tx_id: u64,
    tx_type: u8,
    from_user_id: u64,
    to_user_id: u64,
    amount: i64,
    timestamp: u64,
    status: u8,
    desc_len: u32,
    description: String,
}

impl BinFinanceRecord {
    fn body_len_without_description() -> u32{
        let whole_size = std::mem::size_of::<Self>();
        (whole_size - 8 - std::mem::size_of::<String>()) as u32 // body size = whole_size - size(magic_size) - size(record_size) - size(description)
    }

    fn serialize<Out: Write>(&self, out: &mut Out) -> Result<(), ParsError> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.magic.to_be_bytes());
        buf.extend_from_slice(&self.record_size.to_be_bytes());
        buf.extend_from_slice(&self.tx_id.to_be_bytes());
        buf.extend_from_slice(&self.tx_type.to_be_bytes());
        buf.extend_from_slice(&self.from_user_id.to_be_bytes());
        buf.extend_from_slice(&self.to_user_id.to_be_bytes());
        buf.extend_from_slice(&self.amount.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.status.to_be_bytes());
        buf.extend_from_slice(&self.desc_len.to_be_bytes());
        buf.extend_from_slice(self.description.as_bytes());
        out.write_all(&buf)?;
        Ok(())
    }

    fn deserialize<In: Read>(input: &mut BufReader<In>) -> Result<Self, ParsError> {
        let magic = read_u32(input)?;
        if magic != MAGIC {
            return Err(ParsError::WrongFormat(format!{"Неверный magic: {magic}"}));
        }

        let record_size = read_u32(input)?;
        let mut buf = vec![0u8; record_size as usize];
        input.read_exact(&mut buf)?;

        let tx_id = read_u64(input)?;
        let tx_type = read_u8(input)?;
        let from_user_id = read_u64(input)?;
        let to_user_id = read_u64(input)?;
        let amount = read_i64(input)?;
        let timestamp = read_u64(input)?;
        let status = read_u8(input)?;
        let desc_len = read_u32(input)?;

        let mut desc_buf = vec![0u8; desc_len as usize];
        input.read_exact(&mut desc_buf)?;
        let description = std::str::from_utf8(&desc_buf)?;

        Ok(Self{
            magic,
            record_size,
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            desc_len,
            description: description.to_owned(),
        })
    }
}

pub struct BinReader<In: Read>{
    stream: BufReader<In>,
}

impl <In: Read> BinReader<In>{
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        let record = match BinFinanceRecord::deserialize(&mut self.stream){
            Ok(val) => val,
            Err(e) => {
                if let ParsError::EndOfStream = e {
                    return Ok(None);
                }else{
                    return Err(ParsError::from(e));
                }
            }
        };

        let tx_type = match record.tx_type {
            0 => TxType::Deposit,
            1 => TxType::Transfer,
            2 => TxType::Withdrawal,
            _ => {
                return Err(ParsError::WrongFormat(format!("Wrong tx_type: {}", record.tx_type)));
            }
        };
        let status = match record.status {
            0 => TxStatus::Success,
            1 => TxStatus::Failure,
            2 => TxStatus::Pending,
            _ => {
                return Err(ParsError::WrongFormat(format!("Wrong status: {}", record.status)));
            }
        };

        let timestamp = if let Some(val) = DateTime::from_timestamp_millis(record.timestamp as i64){
            val
        }else{
            return Err(ParsError::WrongFormat(format!("Wrong timestamp: {}", record.timestamp)));
        };

        Ok(Some(FinanceData {
            tx_id: record.tx_id,
            from_user_id: record.from_user_id,
            tx_type,
            to_user_id: record.to_user_id,
            amount: record.amount,
            timestamp,
            status,
            description: record.description.to_owned(),
        }))
    }
}

pub struct BinWriter<Out: Write>{
    stream: Out,
}

impl<Out: Write> BinWriter<Out>{
    pub fn new(stream: Out) -> Result<Self, ParsError>{
        Ok(Self{
            stream,
        })
    }

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError>{
        let tx_type = match data.tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        } as u8;

        let status = match data.status {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        } as u8;

        let timestamp = data.timestamp.timestamp_millis() as u64;

        let record_size = BinFinanceRecord::body_len_without_description() + data.description.len() as u32;
        let record =
        BinFinanceRecord {
            magic: MAGIC,
            record_size,
            tx_id: data.tx_id,
            tx_type,
            from_user_id: data.from_user_id,
            to_user_id: data.to_user_id,
            amount: data.amount,
            timestamp,
            status,
            desc_len: data.description.len() as u32,
            description: data.description.to_owned(),
        };

        record.serialize(&mut self.stream)?;
        Ok(())
    }
}
