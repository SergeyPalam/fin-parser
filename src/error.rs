use std::io;
use thiserror::Error;

/// Класс описания ошибок библиотеки парсинга.

#[derive(Error, Debug)]
pub enum ParsError {
    /// Ошибка ввода-вывода с текстовым описанием
    #[error("Ошибка ввода-вывода: {0}")]
    IoError(String),
    /// Ошибка, указывающая на неверный формат данных
    #[error("Ошибка формата: {0}")]
    WrongFormat(String),
    /// Конец потока
    #[error("Конец потока")]
    EndOfStream,
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
