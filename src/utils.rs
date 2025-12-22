use super::error::ParsError;
use std::io::Read;

pub fn remove_quotes(input: &str) -> String {
    if input.starts_with('"') && input.ends_with('"') {
        input[1..input.len() - 1].to_string()
    } else {
        input.to_string()
    }
}

pub fn read_byte<In: Read>(stream: &mut In) -> Result<u8, ParsError> {
    let mut buf = [0u8; 1];
    match stream.read(&mut buf) {
        Ok(0) => return Err(ParsError::EndOfStream),
        Err(e) => return Err(e.into()),
        Ok(_) => {}
    }
    Ok(buf[0])
}
