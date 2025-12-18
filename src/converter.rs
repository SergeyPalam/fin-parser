use super::parser::*;
use super::error::ParsError;
use super::csv_findata::CsvFinanceRecord;
use super::text_findata::TextFinanceRecord;
use super::bin_findata::BinFinanceRecord;
use std::io::{Read, Write};


const CSV_FORMAT: &str = "csv";
const TEXT_FORMAT: &str = "text";
const BIN_FORMAT: &str = "bin";

pub enum FinFormat {
    Csv,
    Text,
    Bin,
    Unsupported(String),
}

impl FinFormat {
    pub fn from_str(str_format: &str) -> Self {
        match str_format {
            CSV_FORMAT => Self::Csv,
            TEXT_FORMAT => Self::Text,
            BIN_FORMAT => Self::Bin,
            _ => Self::Unsupported(str_format.to_owned()),
        }
    }
}

pub fn convert<In: Read, Out: Write>(from: &mut In, from_format: FinFormat,
        to: &mut Out, to_format: FinFormat) -> Result<(), ParsError>
{
    loop {
        let fin_data = match from_format {
            FinFormat::Csv => {
                read_fin_data::<CsvFinanceRecord, In>(from)
            }
            FinFormat::Text => {
                read_fin_data::<TextFinanceRecord, In>(from)
            }
            FinFormat::Bin => {
                read_fin_data::<BinFinanceRecord, In>(from)
            }
            FinFormat::Unsupported(val) => {
                return Err(ParsError::WrongFormat(format!("Входящий формат не поддерживается: {val}")));
            }
        }?;

        let fin_data = if let Some(val) = fin_data {
            val
        }else {
            return Ok(());
        };

        match to_format {
            FinFormat::Csv => {
                write_fin_data::<CsvFinanceRecord, Out>(&fin_data, to)?
            }
            FinFormat::Text => {
                write_fin_data::<TextFinanceRecord, Out>(&fin_data, to)?
            }
            FinFormat::Bin => {
                write_fin_data::<BinFinanceRecord, Out>(&fin_data, to)?
            }
            FinFormat::Unsupported(val) => {
                return Err(ParsError::WrongFormat(format!("Исходящий формат не поддерживается: {val}")));
            }
        }
    }
}