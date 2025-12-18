use std::io::{Write, Read};
use super::error::ParsError;
use chrono::{DateTime, Utc};

pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

pub struct FinanceData {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: i64,
    pub timestamp: DateTime<Utc>,
    pub status: TxStatus,
    pub description: String,
}

pub trait Serialize {
    fn serialize<T: Write>(&self, out: &mut T) -> Result<(), ParsError>;
}

pub trait Deserialize : Sized{
    fn deserialize<T: Read>(input: &mut T) -> Result<Self, ParsError>;
}

pub trait ToFinanceData {
    fn to_fin_data(&self) -> Result<FinanceData, ParsError>;
}

pub trait FromFinanceData {
    fn from_fin_data(fin_data: &FinanceData) -> Self;
}

pub fn read_fin_data<T, In>(stream: &mut In) -> Result<Option<FinanceData>, ParsError>
    where 
        T: Deserialize + ToFinanceData,
        In: Read
{
    let record = match T::deserialize(stream){
        Ok(val) => val,
        Err(e) => {
            if let ParsError::EndOfStream = e {
                return Ok(None);
            }else{
                return Err(ParsError::from(e));
            }
        }
    };
    record.to_fin_data().map(|fin_data|{
        Some(fin_data)
    })
}

pub fn write_fin_data<T, Out>(data: &FinanceData, stream: &mut Out) -> Result<(), ParsError>
    where
        T: Serialize + FromFinanceData,
        Out: Write
{
    let record = T::from_fin_data(data);
    record.serialize(stream)
}
