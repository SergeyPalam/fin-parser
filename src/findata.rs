use super::parser::*;
use super::error::ParsError;
use super::bin_findata::{BinReader, BinWriter};
use super::csv_findata::{CsvReader, CsvWriter};
use super::text_findata::{TextReader, TextWriter};

use std::io::{Read, Write};

const CSV_FORMAT: &str = "csv";
const TEXT_FORMAT: &str = "text";
const BIN_FORMAT: &str = "bin";

pub enum FinReader<In: Read>{
    Csv(CsvReader<In>),
    Text(TextReader<In>),
    Bin(BinReader<In>),
    Unsupported(String),
}

impl<In: Read> FinReader<In>{
    pub fn new(stream: In, fin_format: &str) -> Result<Self, ParsError> {
        let res =
        match fin_format {
            CSV_FORMAT => Self::Csv(CsvReader::new(stream)?),
            TEXT_FORMAT => Self::Text(TextReader::new(stream)?),
            BIN_FORMAT => Self::Bin(BinReader::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        match self {
            Self::Csv(csv_reader) => {
                csv_reader.read_fin_data()
            }
            Self::Text(text_reader) => {
                text_reader.read_fin_data()
            }
            Self::Bin(bin_reader) => {
                bin_reader.read_fin_data()
            }
            Self::Unsupported(err) => {
                return Err(ParsError::WrongFormat(err.to_owned()));
            }
        }
    }
}

pub enum FinWriter<Out: Write>{
    Csv(CsvWriter<Out>),
    Text(TextWriter<Out>),
    Bin(BinWriter<Out>),
    Unsupported(String),
}

impl<Out: Write> FinWriter<Out>{
    pub fn new(stream: Out, fin_format: &str) -> Result<Self, ParsError> {
        let res =
        match fin_format {
            CSV_FORMAT => Self::Csv(CsvWriter::new(stream)?),
            TEXT_FORMAT => Self::Text(TextWriter::new(stream)?),
            BIN_FORMAT => Self::Bin(BinWriter::new(stream)?),
            _ => Self::Unsupported(fin_format.to_owned()),
        };
        Ok(res)
    }

    pub fn write_fin_data(&mut self, fin_data: &FinanceData) -> Result<(), ParsError> {
        match self {
            Self::Csv(csv_writer) => {
                csv_writer.write_fin_data(fin_data)
            }
            Self::Text(text_writer) => {
                text_writer.write_fin_data(fin_data)
            }
            Self::Bin(bin_writer) => {
                bin_writer.write_fin_data(fin_data)
            }
            Self::Unsupported(err) => {
                return Err(ParsError::WrongFormat(err.to_owned()));
            }
        }
    }
}
