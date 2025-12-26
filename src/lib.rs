//! # fin-parser
//! > Библиотека для чтения-записи транзакций в различных форматах

//! ## About

//! Библиотека для чтения и записи транзакций в форматах bin, csv, text.

#![warn(missing_docs)]
mod bin_format;
mod constants;
mod csv_format;
/// Ошибки в системе
pub mod error;
mod text_format;
/// Транзакция
pub mod transaction;
/// Чтение-запись транзакций
pub mod tx_format;
mod utils;
