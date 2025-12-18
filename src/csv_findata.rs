use super::parser::*;
use super::error::ParsError;
use std::io::{Read, Write, Cursor};
use chrono::DateTime;

pub struct CsvFinanceRecord {
    
}

impl Serialize for CsvFinanceRecord {
    fn serialize<T: Write>(&self, out: &mut T) -> Result<(), ParsError> {
        todo!();
    }
}

impl Deserialize for CsvFinanceRecord {
    fn deserialize<T: Read>(input: &mut T) -> Result<Self, ParsError> {
        todo!();
    }
}
    
impl ToFinanceData for CsvFinanceRecord {
    fn to_fin_data(&self) -> Result<FinanceData, ParsError> {
        todo!();
    }
}

impl FromFinanceData for CsvFinanceRecord {
    fn from_fin_data(fin_data: &FinanceData) -> Self {
        todo!();
    }
}
