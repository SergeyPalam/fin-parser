use super::parser::*;
use super::error::ParsError;
use std::io::{Read, Write, Cursor};
use chrono::DateTime;

pub struct TextFinanceRecord {
    
}

impl Serialize for TextFinanceRecord {
    fn serialize<T: Write>(&self, out: &mut T) -> Result<(), ParsError> {
        todo!();
    }
}

impl Deserialize for TextFinanceRecord {
    fn deserialize<T: Read>(input: &mut T) -> Result<Self, ParsError> {
        todo!();
    }
}
    
impl ToFinanceData for TextFinanceRecord {
    fn to_fin_data(&self) -> Result<FinanceData, ParsError> {
        todo!();
    }
}

impl FromFinanceData for TextFinanceRecord {
    fn from_fin_data(fin_data: &FinanceData) -> Self {
        todo!();
    }
}
