use clap::Parser;
use fin_parser::finance_format::{FinReader, FinWriter};
use std::fs::File;

#[derive(Parser)]
#[command(name = "YpbComparer")]
#[command(version = "1.0")]
#[command(about = "Утилита для сравнения файлов транзакций")]
struct Args {
    /// Путь первого файла
    #[arg(long, value_name = "FILE")]
    lhs_file: String,

    /// Формат первого
    #[arg(long, value_name = "bin | csv | text")]
    lhs_format: String,

    /// Путь второго файла
    #[arg(long, value_name = "FILE")]
    rhs_file: String,

    /// Формат второго файла
    #[arg(long, value_name = "bin | csv | text")]
    rhs_format: String,
}

fn main() {
    let args = Args::parse();
    let lhs_file = match File::open(args.lhs_file) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно открыть файл: {e}");
            return;
        }
    };

    let mut lhs_reader = match FinReader::new(lhs_file, &args.lhs_format) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно создать парсер: {e}");
            return;
        }
    };

    let rhs_file = match File::open(args.rhs_file) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно открыть файл: {e}");
            return;
        }
    };

    let mut rhs_reader = match FinReader::new(rhs_file, &args.rhs_format) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно создать парсер: {e}");
            return;
        }
    };

    loop {
        let lhs_fin_data = lhs_reader.read_fin_data().expect("Ошибка чтения данных");
        let rhs_fin_data = rhs_reader.read_fin_data().expect("Ошибка чтения данных");
        if lhs_fin_data.is_none() && rhs_fin_data.is_none() {
            break;
        }

        if let Some((lhs, rhs)) = lhs_fin_data.zip(rhs_fin_data) {
            if lhs != rhs {
                println!("Записи содержат разные транзакции");
            }
        } else {
            println!("Записи разного размера");
            return;
        }
    }

    println!("Записи идентичны");
}
