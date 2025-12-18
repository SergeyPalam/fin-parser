use std::fmt::Display;
use std::io;

#[derive(Debug)]
pub enum ParsError {
    IoError(String),
    WrongFormat(String),
    EndOfStream,
}

impl Display for ParsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsError::IoError(msg) =>{
                write!(f, "Ошибка ввода вывода: {msg}")
            }
            ParsError::WrongFormat(msg) => {
                write!(f, "Неверный формат: {msg}")
            }
            ParsError::EndOfStream => {
                write!(f, "Конец потока")
            }
            _ => write!(f, "Неизвестная ошибка"),
        }
    }
}

impl From<io::Error> for ParsError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::UnexpectedEof => ParsError::EndOfStream,
            _ => Self::IoError(format!("{e}")),
        }
    }
}

impl From<std::str::Utf8Error> for ParsError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::WrongFormat(format!("{e}"))
    }
}
