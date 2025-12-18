use std::env;
use std::fs::File;
use std::path::Path;
use fin_parser::converter::convert;

fn main() {
    // Получаем аргументы командной строки
    let args: Vec<String> = env::args().collect();

    // Если аргументов недостаточно, показываем справку
    if args.len() != 5 {
        println!("Использование:");
        println!("  <from_path> <from_format> <to_path> <to_format>");
        return;
    }

    let from_path = Path::new(&args[1]);
    let from_format = args[2].to_string();
    let to_path = Path::new(&args[3]);
    let to_format = args[4].to_string();

    let in_file =
    match File::open(from_path){
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно открыть файл: {e}");
            return;
        }
    };

    let out_file =
    match File::create(to_path){
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно создать файл: {e}");
            return;
        }
    };

    if let Err(e) = convert(in_file, &from_format, out_file, &to_format) {
        eprintln!("Ошибка конвертации форматов: {e}");
        return;
    }

    println!("Конвертация выполнена");
}