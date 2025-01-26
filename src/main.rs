mod io;
mod rle;
mod lz77;
mod lz4;
mod processing;
mod lzw;
mod huffman;

use std::env;
use std::time::Instant;
use processing::Algorithm;
use std::fs;
use std::path::Path;
use io::{read_directory, write_directory, pack_directory, unpack_directory, DirEntry};


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 || args.len() > 6 {
        eprintln!("Usage: rle_archiver <compress|decompress> <algorithm> <input_file> <output_file> [mode]");
        return;
    }

    let command = &args[1];
    let algorithm_str = &args[2];
    let input_file = &args[3];
    let output_file = &args[4];

    let use_multithreading = if args.len() == 6 {
        match args[5].as_str() {
            "single" => false,
            "multi" => true,
            _ => {
                eprintln!("Invalid mode. Use 'single' or 'multi'.");
                return;
            }
        }
    } else {
        false
    };

    let algorithm = match algorithm_str.as_str() {
        "rle" => Algorithm::Rle,
        "lz77" => Algorithm::Lz77,
        "lz4" => Algorithm::Lz4,
        "lzw" => Algorithm::Lzw, 
        "hf" => Algorithm::Hf, 
        _ => {
            eprintln!("Unknown algorithm: {}", algorithm_str);
            return;
        }
    };

    let input_path = Path::new(input_file);
    let start_time = Instant::now();

    let output_data = if input_path.is_dir() && command == "compress" {
        let entries = read_directory(input_file).expect("Не удалось прочитать директорию");
        let packed = pack_directory(&entries);
        processing::compress(&packed, algorithm, use_multithreading)
    } else if command == "decompress" && !use_multithreading {
        let input = io::read_file(input_file).expect("Failed to read input file");
        let decompressed = processing::decompress(&input, algorithm, false);
        let entries = unpack_directory(&decompressed);
        fs::create_dir_all(output_file).expect("Не удалось создать директорию для вывода");
        write_directory(output_file, &entries).expect("Не удалось записать директорию");
        Vec::new()
    } else {
        let input = io::read_file(input_file).expect("Failed to read input file");
        match command.as_str() {
            "compress" => processing::compress(&input, algorithm, use_multithreading),
            "decompress" => {
                let result = processing::decompress(&input, algorithm, use_multithreading);
                if result.is_empty() {
                    eprintln!("Decompression failed.");
                    return;
                }
                result
            },
            _ => {
                eprintln!("Invalid command. Use 'compress' or 'decompress'.");
                return;
            }
        }
    };

    let duration = start_time.elapsed();
    if !output_data.is_empty() {
        io::write_file(output_file, &output_data).expect("Failed to write output file");
    }
    println!("Операция завершена успешно.");
    println!("Затраченное время: {:.2?}", duration);
}