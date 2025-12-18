use super::parser::*;
use super::error::ParsError;
use std::io::{Read, Write, Cursor};
use chrono::DateTime;

struct TextFinanceRecord {
}

impl TextFinanceRecord {
    fn serialize<Out: Write>(&self, out: &mut Out) -> Result<(), ParsError> {
        todo!();
    }

    fn deserialize<In: Read>(input: &mut In) -> Result<Self, ParsError> {
        todo!();
    }
}

pub struct TextReader<In: Read>{
    stream: In,
}

impl <In: Read> TextReader<In>{
    pub fn new(stream: In) -> Result<Self, ParsError> {
        todo!();
    }

    pub fn read_fin_data(&mut self) -> Result<Option<FinanceData>, ParsError> {
        todo!();
    }
}

pub struct TextWriter<Out: Write>{
    stream: Out,
}

impl<Out: Write> TextWriter<Out>{
    pub fn new(stream: Out) -> Result<Self, ParsError>{
        todo!()
    }

    pub fn write_fin_data(&mut self, data: &FinanceData) -> Result<(), ParsError>{
        todo!();
    }
}
