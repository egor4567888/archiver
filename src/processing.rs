use crate::rle;
use crate::lz77;
use crate::lz4;
use crate::lzw;
use std::thread;

pub enum Algorithm {
    Rle,
    Lz77,
    Lz4,
    Lzw,
}


impl Clone for Algorithm {
    fn clone(&self) -> Self {
        match self {
            Algorithm::Rle => Algorithm::Rle,
            Algorithm::Lz77 => Algorithm::Lz77,
            Algorithm::Lz4 => Algorithm::Lz4,
            Algorithm::Lzw => Algorithm::Lzw, 
        }
    }
}


pub fn compress(input: &[u8], algorithm: Algorithm, use_multithreading: bool) -> Vec<u8> {
    if use_multithreading {

        let num_threads = 4;
        let chunk_size = (input.len() + num_threads - 1) / num_threads;

        let mut handles = Vec::new();

        for chunk in input.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let algo = algorithm.clone();
            let handle = thread::spawn(move || {
                match algo {
                    Algorithm::Rle => rle::compress(&chunk),
                    Algorithm::Lz77 => lz77::compress(&chunk),
                    Algorithm::Lz4 => lz4::compress(&chunk),
                    Algorithm::Lzw => lzw::compress(&chunk), 
                }
            });
            handles.push(handle);
        }

        let mut compressed = Vec::new();
        for handle in handles {
            let data = handle.join().expect("Thread failed");
            compressed.extend(data);
        }

        compressed
    } else {
        match algorithm {
            Algorithm::Rle => rle::compress(input),
            Algorithm::Lz77 => lz77::compress(input),
            Algorithm::Lz4 => lz4::compress(input),
            Algorithm::Lzw => lzw::compress(input), 
        }
    }
}


pub fn decompress(input: &[u8], algorithm: Algorithm, use_multithreading: bool) -> Vec<u8> {
    if use_multithreading {
        eprintln!("Multithreading not supported for decompression.");
        match algorithm {
            Algorithm::Rle => rle::decompress(input),
            Algorithm::Lz77 => lz77::decompress(input),
            Algorithm::Lz4 => lz4::decompress(input),
            Algorithm::Lzw => lzw::decompress(input), 
        }
    } else {
        match algorithm {
            Algorithm::Rle => rle::decompress(input),
            Algorithm::Lz77 => lz77::decompress(input),
            Algorithm::Lz4 => lz4::decompress(input),
            Algorithm::Lzw => lzw::decompress(input),
        }
    }
}