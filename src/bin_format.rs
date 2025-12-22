use super::error::ParsError;
use super::transaction::*;
use super::utils::remove_quotes;
use chrono::DateTime;
use std::io::{BufReader, Read, Write};

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

#[derive(Eq, PartialEq, Debug)]
struct BinTxRecord {
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

impl BinTxRecord {
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
            return Err(ParsError::WrongFormat(format! {"Неверный magic: {magic}"}));
        }
        let record_size = read_u32(input)?;

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

        Ok(Self {
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

    fn to_transaction(&self) -> Result<Transaction, ParsError> {
        let tx_type = match self.tx_type {
            0 => TxType::Deposit,
            1 => TxType::Transfer,
            2 => TxType::Withdrawal,
            _ => {
                return Err(ParsError::WrongFormat(format!(
                    "Wrong tx_type: {}",
                    self.tx_type
                )));
            }
        };
        let status = match self.status {
            0 => TxStatus::Success,
            1 => TxStatus::Failure,
            2 => TxStatus::Pending,
            _ => {
                return Err(ParsError::WrongFormat(format!(
                    "Wrong status: {}",
                    self.status
                )));
            }
        };

        let timestamp = if let Some(val) = DateTime::from_timestamp_millis(self.timestamp as i64) {
            val
        } else {
            return Err(ParsError::WrongFormat(format!(
                "Wrong timestamp: {}",
                self.timestamp
            )));
        };

        if !(self.description.starts_with('"') && self.description.ends_with('"')) {
            return Err(ParsError::WrongFormat(format!(
                "Wrong description: {}",
                self.description
            )));
        }

        Ok(Transaction {
            tx_id: self.tx_id,
            from_user_id: self.from_user_id,
            tx_type,
            to_user_id: self.to_user_id,
            amount: self.amount,
            timestamp,
            status,
            description: remove_quotes(&self.description),
        })
    }

    fn from_transaction(tx: &Transaction) -> Self {
        let tx_type = match tx.tx_type {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
        } as u8;

        let status = match tx.status {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        } as u8;

        let timestamp = tx.timestamp.timestamp_millis() as u64;

        let description = format!("\"{}\"", tx.description);
        let desc_len = description.len() as u32;
        let record_size = std::mem::size_of_val(&tx.tx_id)
            + std::mem::size_of_val(&tx_type)
            + std::mem::size_of_val(&tx.from_user_id)
            + std::mem::size_of_val(&tx.to_user_id)
            + std::mem::size_of_val(&tx.amount)
            + std::mem::size_of_val(&timestamp)
            + std::mem::size_of_val(&status)
            + std::mem::size_of_val(&desc_len)
            + description.len();
        Self {
            magic: MAGIC,
            record_size: record_size as u32,
            tx_id: tx.tx_id,
            tx_type,
            from_user_id: tx.from_user_id,
            to_user_id: tx.to_user_id,
            amount: tx.amount,
            timestamp,
            status,
            desc_len,
            description,
        }
    }
}

pub struct BinTxReader<In: Read> {
    stream: BufReader<In>,
}

impl<In: Read> BinTxReader<In> {
    pub fn new(stream: In) -> Result<Self, ParsError> {
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    pub fn read_transaction(&mut self) -> Result<Option<Transaction>, ParsError> {
        let record = match BinTxRecord::deserialize(&mut self.stream) {
            Ok(val) => val,
            Err(e) => {
                if let ParsError::EndOfStream = e {
                    return Ok(None);
                } else {
                    return Err(ParsError::from(e));
                }
            }
        };

        Ok(Some(record.to_transaction()?))
    }
}

pub struct BinTxWriter<Out: Write> {
    stream: Out,
}

impl<Out: Write> BinTxWriter<Out> {
    pub fn new(stream: Out) -> Result<Self, ParsError> {
        Ok(Self { stream })
    }

    pub fn write_transaction(&mut self, data: &Transaction) -> Result<(), ParsError> {
        let record = BinTxRecord::from_transaction(&data);
        record.serialize(&mut self.stream)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use std::io::Cursor;

    const EXPECTED_BIN: &[u8] = &hex!(
        "
    59 50 42 4e 00 00 00 3f 00 03 8d 7e a4 c6 80 00
    00 00 00 00 00 00 00 00 00 7f ff ff ff ff ff ff
    ff 00 00 00 00 00 00 00 64 00 00 01 7c 38 94 fa
    60 01 00 00 00 11 22 52 65 63 6f 72 64 20 6e 75
    6d 62 65 72 20 31 22
    "
    );

    const EXPECTED_BIN_MULT: &[u8] = &hex!(
        "
        59 50 42 4e 00 00 00 3f 00 03 8d 7e a4 c6 80 00
        00 00 00 00 00 00 00 00 00 7f ff ff ff ff ff ff
        ff 00 00 00 00 00 00 00 64 00 00 01 7c 38 94 fa
        60 01 00 00 00 11 22 52 65 63 6f 72 64 20 6e 75
        6d 62 65 72 20 31 22 59 50 42 4e 00 00 00 3f 00
        03 8d 7e a4 c6 80 01 01 7f ff ff ff ff ff ff ff
        7f ff ff ff ff ff ff ff 00 00 00 00 00 00 00 c8
        00 00 01 7c 38 95 e4 c0 02 00 00 00 11 22 52 65
        63 6f 72 64 20 6e 75 6d 62 65 72 20 32 22
    "
    );

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

    fn bin_record_for_test() -> BinTxRecord {
        BinTxRecord {
            magic: MAGIC,
            record_size: (EXPECTED_BIN.len() - 8) as u32,
            tx_id: 1000000000000000,
            tx_type: 0,
            from_user_id: 0,
            to_user_id: 9223372036854775807,
            amount: 100,
            timestamp: 1633036860000,
            status: 1,
            desc_len: 17,
            description: "\"Record number 1\"".to_owned(),
        }
    }

    #[test]
    fn test_bin_from_transaction() {
        let tx = tx1_for_test();
        let expected = bin_record_for_test();
        let record = BinTxRecord::from_transaction(&tx);

        assert_eq!(record, expected);
    }

    #[test]
    fn test_bin_to_transaction() {
        let bin_record = bin_record_for_test();
        let expected = tx1_for_test();
        let tx = bin_record.to_transaction().unwrap();

        assert_eq!(tx, expected);
    }

    #[test]
    fn test_serialize_bin_record() {
        let record = bin_record_for_test();
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        record.serialize(&mut cursor).unwrap();

        assert_eq!(cursor.get_ref(), EXPECTED_BIN);
    }

    #[test]
    fn test_deserialize_bin_record() {
        let expected = bin_record_for_test();
        let mut buf = BufReader::new(Cursor::new(EXPECTED_BIN));
        let record = BinTxRecord::deserialize(&mut buf).unwrap();

        assert_eq!(record, expected);
    }

    #[test]
    fn test_bin_reader() {
        let stream = Cursor::new(EXPECTED_BIN_MULT);
        let mut bin_reader = BinTxReader::new(stream).unwrap();

        let mut fin_info = Vec::new();
        while let Some(tx) = bin_reader.read_transaction().unwrap() {
            fin_info.push(tx);
        }

        assert_eq!(fin_info.len(), 2);
        assert_eq!(fin_info[0], tx1_for_test());
        assert_eq!(fin_info[1], tx2_for_test());
    }

    #[test]
    fn test_bin_writer() {
        let buf = Vec::new();
        let stream = Cursor::new(buf);
        let mut bin_writer = BinTxWriter::new(stream).unwrap();

        bin_writer.write_transaction(&tx1_for_test()).unwrap();
        bin_writer.write_transaction(&tx2_for_test()).unwrap();
        assert_eq!(bin_writer.stream.get_ref(), EXPECTED_BIN_MULT);
    }
}
