use super::parser::*;
use super::error::ParsError;
use std::io::{Read, Write, Cursor};
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

pub struct BinFinanceRecord {
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
}
impl Serialize for BinFinanceRecord {
    fn serialize<T: Write>(&self, out: &mut T) -> Result<(), ParsError> {
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
}

impl Deserialize for BinFinanceRecord {
    fn deserialize<T: Read>(input: &mut T) -> Result<Self, ParsError> {
        let magic = read_u32(input)?;
        if magic != MAGIC {
            return Err(ParsError::WrongFormat(format!{"Неверный magic: {magic}"}));
        }

        let record_size = read_u32(input)?;
        let mut buf = Vec::<u8>::with_capacity(record_size as usize);
        input.read_exact(&mut buf)?;

        let mut cursor = Cursor::new(buf);
        let tx_id = read_u64(input)?;
        let tx_type = read_u8(input)?;
        let from_user_id = read_u64(input)?;
        let to_user_id = read_u64(input)?;
        let amount = read_i64(input)?;
        let timestamp = read_u64(input)?;
        let status = read_u8(input)?;
        let desc_len = read_u32(input)?;

        let mut desk_buf = Vec::<u8>::with_capacity(desc_len as usize);
        cursor.read_exact(&mut desk_buf)?;
        let description = std::str::from_utf8(&desk_buf)?;

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
    
impl ToFinanceData for BinFinanceRecord {
    fn to_fin_data(&self) -> Result<FinanceData, ParsError> {
        let tx_type = match self.tx_type {
            0 => TxType::Deposit,
            1 => TxType::Transfer,
            2 => TxType::Withdrawal,
            _ => {
                return Err(ParsError::WrongFormat(format!("Wrong tx_type: {}", self.tx_type)));
            }
        };
        let status = match self.status {
            0 => TxStatus::Success,
            1 => TxStatus::Failure,
            2 => TxStatus::Pending,
            _ => {
                return Err(ParsError::WrongFormat(format!("Wrong status: {}", self.status)));
            }
        };

        let timestamp = if let Some(val) = DateTime::from_timestamp_millis(self.timestamp as i64){
            val
        }else{
            return Err(ParsError::WrongFormat(format!("Wrong timestamp: {}", self.timestamp)));
        };

        Ok(FinanceData {
            tx_id: self.tx_id,
            from_user_id: self.from_user_id,
            tx_type,
            to_user_id: self.to_user_id,
            amount: self.amount,
            timestamp,
            status,
            description: self.description.to_owned(),
        })
    }
}

impl FromFinanceData for BinFinanceRecord {
    fn from_fin_data(fin_data: &FinanceData) -> Self {
        let tx_type = match fin_data.tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        } as u8;

        let status = match fin_data.status {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        } as u8;

        let timestamp = fin_data.timestamp.timestamp_millis() as u64;

        let record_size = Self::body_len_without_description() + fin_data.description.len() as u32;
        Self {
            magic: MAGIC,
            record_size,
            tx_id: fin_data.tx_id,
            tx_type,
            from_user_id: fin_data.from_user_id,
            to_user_id: fin_data.to_user_id,
            amount: fin_data.amount,
            timestamp,
            status,
            desc_len: fin_data.description.len() as u32,
            description: fin_data.description.to_owned(),
        }
    }
}
