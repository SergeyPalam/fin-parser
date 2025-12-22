use super::bin_format::{BinTxReader, BinTxWriter};
use super::csv_format::{CsvTxReader, CsvTxWriter};
use super::error::ParsError;
use super::text_format::{TextTxReader, TextTxWriter};
use super::transaction::*;

use std::io::{Read, Write};

const CSV_FORMAT: &str = "csv";
const TEXT_FORMAT: &str = "text";
const BIN_FORMAT: &str = "bin";

pub enum TxReader<In: Read> {
    Csv(CsvTxReader<In>),
    Text(TextTxReader<In>),
    Bin(BinTxReader<In>),
    Unsupported(String),
}

impl<In: Read> TxReader<In> {
    pub fn new(stream: In, fin_format: &str) -> Result<Self, ParsError> {
        let res = match fin_format {
            CSV_FORMAT => Self::Csv(CsvTxReader::new(stream)?),
            TEXT_FORMAT => Self::Text(TextTxReader::new(stream)?),
            BIN_FORMAT => Self::Bin(BinTxReader::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

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

pub enum TxWriter<Out: Write> {
    Csv(CsvTxWriter<Out>),
    Text(TextTxWriter<Out>),
    Bin(BinTxWriter<Out>),
    Unsupported(String),
}

impl<Out: Write> TxWriter<Out> {
    pub fn new(stream: Out, fin_format: &str) -> Result<Self, ParsError> {
        let res = match fin_format {
            CSV_FORMAT => Self::Csv(CsvTxWriter::new(stream)?),
            TEXT_FORMAT => Self::Text(TextTxWriter::new(stream)?),
            BIN_FORMAT => Self::Bin(BinTxWriter::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

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
