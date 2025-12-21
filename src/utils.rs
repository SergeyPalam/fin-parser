use super::error::ParsError;

pub fn remove_quotes(input: &str) -> Result<String, ParsError> {
    let res = if let Some(val) = input
        .strip_prefix("\"")
        .map(|val| val.strip_suffix("\""))
        .flatten()
    {
        val.to_owned()
    } else {
        return Err(ParsError::WrongFormat(format!("Неверный формат: {input}")));
    };
    Ok(res)
}
