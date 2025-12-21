use chrono::{DateTime, Utc};

#[derive(Eq, PartialEq, Debug)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Eq, PartialEq, Debug)]
pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

#[derive(Eq, PartialEq, Debug)]
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
