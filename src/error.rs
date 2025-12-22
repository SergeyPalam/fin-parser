use std::fmt::Display;
use std::io;

/// Класс описания ошибок библиотеки парсинга.

#[derive(Debug)]
pub enum ParsError {
    /// Ошибка ввода-вывода с текстовым описанием
    IoError(String),
    /// Ошибка, указывающая на неверный формат данных
    WrongFormat(String),
    /// Конец потока
    EndOfStream,
}

impl Display for ParsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsError::IoError(msg) => {
                write!(f, "Ошибка ввода вывода: {msg}")
            }
            ParsError::WrongFormat(msg) => {
                write!(f, "Неверный формат: {msg}")
            }
            ParsError::EndOfStream => {
                write!(f, "Конец потока")
            }
        }
    }
}

/// Ошибка ввода-вывода io::Error преобразуется по следующим правилам:
///  - io::ErrorKind::UnexpectedEof to ParsError::EndOfStream
///  - Любая другая ошибка io::Error to ParsError::IoError
impl From<io::Error> for ParsError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::UnexpectedEof => ParsError::EndOfStream,
            _ => Self::IoError(format!("{e}")),
        }
    }
}

/// Ошибка, возникающая при парсинге UTF8-строки
impl From<std::str::Utf8Error> for ParsError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::WrongFormat(format!("{e}"))
    }
}

/// Ошибка, возникающая при парсинге целых чисел из строки
impl From<std::num::ParseIntError> for ParsError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::WrongFormat(format!("{e}"))
    }
}
