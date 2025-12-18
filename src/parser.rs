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
