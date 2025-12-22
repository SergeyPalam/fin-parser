use clap::Parser;
use fin_parser::tx_format::{TxReader, TxWriter};
use std::fs::File;

#[derive(Parser)]
#[command(name = "YpbConverter")]
#[command(version = "1.0")]
#[command(about = "Утилита конвертации форматов")]
struct Args {
    /// Путь к входному файлу
    #[arg(long, value_name = "FILE")]
    input_file: String,

    /// Формат входных данных
    #[arg(long, value_name = "bin | csv | text")]
    input_format: String,

    /// Формат выходных данных
    #[arg(long, value_name = "bin | csv | text")]
    output_format: String,
}

fn main() {
    let args = Args::parse();
    let input_file = match File::open(args.input_file) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно открыть файл: {e}");
            return;
        }
    };

    let mut reader = match TxReader::new(input_file, &args.input_format) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно создать парсер: {e}");
            return;
        }
    };

    let mut writer = match TxWriter::new(std::io::stdout(), &args.output_format) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно создать парсер для записи: {e}");
            return;
        }
    };

    loop {
        let tx = match reader.read_transaction() {
            Ok(data) => {
                if let Some(val) = data {
                    val
                } else {
                    println!("Файл успешно считан");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Ошибка чтения данных: {e}");
                return;
            }
        };
        if let Err(e) = writer.write_transaction(&tx) {
            eprintln!("Ошибка вывода данных: {e}");
            return;
        }
    }
}
