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
use serde::{Deserialize, Serialize};
use bincode;
use std::path::{Path};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

#[derive(Serialize, Deserialize)]
struct ArchiveData {
    entries: Vec<io::DirEntry>,
}



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

    match command.as_str() {
        "compress" => {
            let entries = io::read_dir_recursive(input_path, input_path).expect("Failed to read path");
            let serialized = bincode::serialize(&ArchiveData { entries })
                .expect("Failed to serialize directory/file data");
            let compressed = processing::compress(&serialized, algorithm, use_multithreading);
            io::write_file(output_file, &compressed).expect("Failed to write output file");
        },
        "decompress" => {
            let compressed_data = io::read_file(input_file).expect("Failed to read input file");
            let decompressed = processing::decompress(&compressed_data, algorithm, use_multithreading);
            if decompressed.is_empty() {
                eprintln!("Decompression failed.");
                return;
            }
            let archive: ArchiveData = bincode::deserialize(&decompressed)
                .expect("Failed to deserialize data");
            if archive.entries.len() == 1 {
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
            eprintln!("Invalid command. Use 'compress' or 'decompress'.");
            return;
        }
    };

    let duration = start_time.elapsed();
    println!("Elapsed time: {:.2?}", duration);
}