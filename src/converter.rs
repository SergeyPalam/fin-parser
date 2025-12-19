use super::error::ParsError;
use super::finance_format::{FinReader, FinWriter};
use std::io::{Read, Write};

pub fn convert<In: Read, Out: Write>(
    from: In,
    from_format: &str,
    to: Out,
    to_format: &str,
) -> Result<(), ParsError> {
    let mut reader = FinReader::new(from, from_format)?;
    let mut writer = FinWriter::new(to, to_format)?;

    loop {
        let fin_data = reader.read_fin_data()?;
        let fin_data = if let Some(val) = fin_data {
            val
        } else {
            return Ok(());
        };
        writer.write_fin_data(&fin_data)?;
    }
}
