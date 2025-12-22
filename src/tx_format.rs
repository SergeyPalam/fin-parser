use super::bin_format::{BinTxReader, BinTxWriter};
use super::csv_format::{CsvTxReader, CsvTxWriter};
use super::error::ParsError;
use super::text_format::{TextTxReader, TextTxWriter};
use super::transaction::*;

use std::io::{Read, Write};

const CSV_FORMAT: &str = "csv";
const TEXT_FORMAT: &str = "text";
const BIN_FORMAT: &str = "bin";

/// # Основной функционал библиотеки,
/// # реализующий методы записи и чтения транзакций в различных форматах
/// ## Example

///```
/// use fin_parser::tx_format::{TxReader, TxWriter};
/// use std::io::Cursor;

/// fn main() {
///     let text_tx = r#"# Record 1 (DEPOSIT)
///     TX_TYPE: DEPOSIT
///     TO_USER_ID: 9223372036854775807
///     FROM_USER_ID: 0
///     TIMESTAMP: 1633036860000
///     DESCRIPTION: "Record number 1"
///     TX_ID: 1000000000000000
///     AMOUNT: 100
///     STATUS: FAILURE
///     "#;

///     let cursor = Cursor::new(text_tx.as_bytes());
///     let mut reader = TxReader::new(cursor, "text").unwrap();
///     let tx = reader.read_transaction().unwrap().unwrap();

///     let mut writer = TxWriter::new(std::io::stdout(), "csv").unwrap();
///     writer.write_transaction(&tx).unwrap();
/// }

///```

/// Обертка над потоком Read, читающая транзакции, записанные в форматах
/// - csv
/// - text
/// - bin
pub enum TxReader<In: Read> {
    Csv(CsvTxReader<In>),
    Text(TextTxReader<In>),
    Bin(BinTxReader<In>),
    Unsupported(String),
}

impl<In: Read> TxReader<In> {
    /// Конструктор, принимающий на вход поток и один из трёх форматов
    /// - csv
    /// - text
    /// - bin
    pub fn new(stream: In, fin_format: &str) -> Result<Self, ParsError> {
        let res = match fin_format {
            CSV_FORMAT => Self::Csv(CsvTxReader::new(stream)?),
            TEXT_FORMAT => Self::Text(TextTxReader::new(stream)?),
            BIN_FORMAT => Self::Bin(BinTxReader::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

    /// Метод чтения одной транзакции. TxReader читает порциями из потока, чтобы не создавать
    /// дополнительную нагрузку на память
    pub fn read_transaction(&mut self) -> Result<Option<Transaction>, ParsError> {
        match self {
            Self::Csv(csv_reader) => csv_reader.read_transaction(),
            Self::Text(text_reader) => text_reader.read_transaction(),
            Self::Bin(bin_reader) => bin_reader.read_transaction(),
            Self::Unsupported(err) => {
                return Err(ParsError::WrongFormat(err.to_owned()));
            }
        }
    }
}

/// Обертка над потоком Write, пишущая транзакции, в форматах
/// - csv
/// - text
/// - bin
pub enum TxWriter<Out: Write> {
    Csv(CsvTxWriter<Out>),
    Text(TextTxWriter<Out>),
    Bin(BinTxWriter<Out>),
    Unsupported(String),
}

impl<Out: Write> TxWriter<Out> {
    /// Конструктор, принимающий на вход поток и один из трёх форматов
    /// - csv
    /// - text
    /// - bin
    pub fn new(stream: Out, fin_format: &str) -> Result<Self, ParsError> {
        let res = match fin_format {
            CSV_FORMAT => Self::Csv(CsvTxWriter::new(stream)?),
            TEXT_FORMAT => Self::Text(TextTxWriter::new(stream)?),
            BIN_FORMAT => Self::Bin(BinTxWriter::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

    /// Метод записи одной транзакции.
    pub fn write_transaction(&mut self, tx: &Transaction) -> Result<(), ParsError> {
        match self {
            Self::Csv(csv_writer) => csv_writer.write_transaction(tx),
            Self::Text(text_writer) => text_writer.write_transaction(tx),
            Self::Bin(bin_writer) => bin_writer.write_transaction(tx),
            Self::Unsupported(err) => {
                return Err(ParsError::WrongFormat(err.to_owned()));
            }
        }
    }
}
