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

/// Тип данных, описывающий информацию о транзакции
#[derive(Eq, PartialEq, Debug)]
pub struct Transaction {
    /// Идентификатор транзакции
    pub tx_id: u64,
    /// Тип транзакции
    pub tx_type: TxType,
    /// Идентификатор инициатора транзакции
    pub from_user_id: u64,
    /// Идентификатор получателя транзакции
    pub to_user_id: u64,
    /// Сумма транзакции
    pub amount: i64,
    /// Время транзакции
    pub timestamp: DateTime<Utc>,
    /// Статус транзакции
    pub status: TxStatus,
    /// Описание транзакции
    pub description: String,
}
