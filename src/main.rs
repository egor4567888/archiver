//! Основной модуль архиватора, отвечающий за сжатие и распаковку файлов с использованием различных алгоритмов.
mod io;
mod rle;
mod lz77;
mod lz4;
mod processing;
mod lzw;
mod huffman;

use std::time::Instant;
use processing::Algorithm;
use serde::{Deserialize, Serialize};
use std::path::Path;

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use clap::{Command, Arg, ArgAction};
use log::error;

/// Структура для хранения данных архива.
#[derive(Serialize, Deserialize)]
struct ArchiveData {
    /// Список записей директории.
    entries: Vec<io::DirEntry>,
}


/// Главная функция приложения.
/// Инициализирует и настраивает команду rle_archiver с различными аргументами.
    ///
    /// ## Аргументы
    ///
    /// - `compress` (`-c`): Сжимает файлы. Не может использоваться вместе с `decompress`. Обязателен, если не указан `decompress`.
    /// - `decompress` (`-d`): Распаковывает файлы. Не может использоваться вместе с `compress`. Обязателен, если не указан `compress`.
    /// - `algorithm`: Выбор алгоритма сжатия. Обязательный аргумент.
    /// - `input`: Входной файл для обработки. Обязательный аргумент.
    /// - `output`: Выходной файл. Обязательный аргумент.
    /// - `multithread` (`-m`): Включает многопоточную обработку.
fn main() {
    
    
    // Определение аргументов командной строки
    let matches = Command::new("rle_archiver")
        .version("1.0")
        .author("Your Name <youremail@example.com>")
        .about("Compresses and decompresses files using various algorithms")
        .arg(Arg::new("compress")
            .short('c')
            .conflicts_with("decompress")
            .help("Compress files")
            .required_unless_present("decompress")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("decompress")
            .short('d')
            .conflicts_with("compress")
            .help("Decompress files")
            .required_unless_present("compress")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("algorithm")
            .short('a')
            .help("Compression algorithm to use")
            .required(true)
            .num_args(1)) 
        .arg(Arg::new("input")
            .short('i')
            .help("Input file to process")
            .required(true)
            .num_args(1)) 
        .arg(Arg::new("output")
            .short('o')
            .help("Output file")
            .required(true)
            .num_args(1))
        .arg(Arg::new("multithread")
            .short('m')
            .help("Enable multithreading")
            .action(ArgAction::SetTrue))
        .get_matches();

    // Определение команды (сжатие или распаковка)
    let command = if matches.get_flag("compress") {
        "compress"
    } else {
        "decompress"
    };

    // Извлечение значений аргументов
    let algorithm_str = matches.get_one::<String>("algorithm").unwrap();
    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();

    let use_multithreading = matches.get_flag("multithread");

    // Определение алгоритма на основе аргумента
    let algorithm = match algorithm_str.as_str() {
        "rle" => Algorithm::Rle,
        "lz77" => Algorithm::Lz77,
        "lz4" => Algorithm::Lz4,
        "lzw" => Algorithm::Lzw,
        "hf" => Algorithm::Hf,
        _ => {
            error!("Неподдерживаемый алгоритм: {}", algorithm_str);
            std::process::exit(1);
        }
    };

    let input_path = Path::new(input_file);
    let start_time = Instant::now();


    // Выполнение команды
    match command {
        "compress" => {
            // Чтение директории и сериализация данных
            let entries = io::read_dir_recursive(input_path, input_path).expect("Failed to read path");
            let serialized = io::archive_data_to_bytes(&ArchiveData { entries });
            
            // Сжатие данных и запись в выходной файл
            let compressed = processing::compress(&serialized, algorithm, use_multithreading);
            io::write_file(output_file, &compressed).expect("Failed to write output file");
        },
        "decompress" => {
            // Чтение сжатого файла и его распаковка
            let compressed_data = io::read_file(input_file).expect("Failed to read input file");
            let decompressed = processing::decompress(&compressed_data, algorithm, use_multithreading);
            if decompressed.is_empty() {
                error!("Decompression failed.");
                return;
            }
            // Десериализация данных и запись в выходной файл
            let archive: ArchiveData = io::bytes_to_archive_data(&decompressed)
                .expect("Failed to deserialize data");
            if archive.entries.len() == 1 { // Обработка единичных файлов
                let e = &archive.entries[0];
                let mut file = std::fs::File::create(output_file)
                    .expect("Failed to create single output file");
                file.write_all(&e.data).expect("Failed to write data");
                std::fs::set_permissions(output_file, std::fs::Permissions::from_mode(e.permissions))
                    .expect("Failed to set permissions");
            } else {
                io::write_dir_entries(&archive.entries, Path::new(output_file))
                    .expect("Failed to write directory entries");
            }
        },
        _ => {
            error!("Invalid command. Use 'compress' or 'decompress'.");
            return;
        }
    };

    // Вывод времени выполнения
    let duration = start_time.elapsed();
    println!("Program executed successfully.");
    println!("Elapsed time: {:.2?}", duration);
}