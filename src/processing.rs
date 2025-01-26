//! Модуль для сжатия и распаковки данных с использованием различных алгоритмов.
//!
//! Этот модуль предоставляет функции для сжатия и распаковки данных с использованием различных алгоритмов, таких как RLE, LZ77, LZ4, LZW и алгоритм Хаффмана. 
//! Также поддерживается многопоточное сжатие для некоторых алгоритмов.
//! 
use crate::rle;
use crate::lz77;
use crate::lz4;
use crate::lzw;
use crate::huffman;
use std::thread;
use log::error;
#[derive(PartialEq)]
pub enum Algorithm {
    /// Алгоритм RLE (Run-Length Encoding) для сжатия повторяющихся данных.
    Rle,
    /// Алгоритм LZ77 для сжатия данных путем поиска повторяющихся последовательностей.
    Lz77,
    /// Алгоритм LZ4 для быстрого сжатия и распаковки данных.
    Lz4,
    /// Алгоритм LZW (Lempel-Ziv-Welch) для сжатия данных.
    Lzw,
    /// Алгоритм Хаффмана для сжатия данных с использованием кодирования Хаффмана.
    Hf,
}

/// Реализация клонирования для перечисления `Algorithm`.
impl Clone for Algorithm {
    /// Создает копию текущего экземпляра `Algorithm`.
    ///
    /// # Возвращаемое значение
    ///
    /// Новый экземпляр `Algorithm`, соответствующий текущему.
    fn clone(&self) -> Self {
        match self {
            Algorithm::Rle => Algorithm::Rle,
            Algorithm::Lz77 => Algorithm::Lz77,
            Algorithm::Lz4 => Algorithm::Lz4,
            Algorithm::Lzw => Algorithm::Lzw, 
            Algorithm::Hf => Algorithm::Hf, 
        }
    }
}

/// Сжимает входные данные с использованием выбранного алгоритма.
/// 
/// Если `use_multithreading` установлено в `true`, сжатие выполняется в многопоточном режиме.
/// 
/// # Аргументы
/// 
/// * `input` - Срез байтов, содержащий исходные данные для сжатия.
/// * `algorithm` - Выбранный алгоритм сжатия.
/// * `use_multithreading` - Флаг, указывающий использовать ли многопоточность.
/// 
/// # Возвращает
/// 
/// Вектор байтов, содержащий сжатые данные.
/// # Примечания
/// 
/// При попытке использовать многопоточность для lzw или алгоритма Хаффмена будет использован однопоточный режим.
pub fn compress(input: &[u8], algorithm: Algorithm, use_multithreading: bool) -> Vec<u8> {
    if use_multithreading && (algorithm!=Algorithm::Hf && algorithm!=Algorithm::Lzw) {

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
                    Algorithm::Hf => huffman::compress(&chunk), 
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
            Algorithm::Hf => huffman::compress(input), 
        }
    }
}

/// Распаковывает сжатые данные с использованием выбранного алгоритма.
/// 
/// Если `use_multithreading` установлено в `true`, распаковка выполняется в многопоточном режиме.
/// Однако в текущей реализации многопоточность для распаковки не поддерживается.
/// 
///
/// # Аргументы
/// 
/// * `input` - Срез байтов, содержащий сжатые данные для распаковки.
/// * `algorithm` - Выбранный алгоритм распаковки.
/// * `use_multithreading` - Флаг, указывающий использовать ли многопоточность.
/// 
/// # Возвращает
/// 
/// Вектор байтов, содержащий распакованные данные.
/// 
/// # Примечания
/// 
/// При попытке использовать многопоточность для распаковки будет записано сообщение об ошибке в лог.
pub fn decompress(input: &[u8], algorithm: Algorithm, use_multithreading: bool) -> Vec<u8> {
    if use_multithreading {
        error!("Multithreading not supported for decompression.");
        match algorithm {
            Algorithm::Rle => rle::decompress(input),
            Algorithm::Lz77 => lz77::decompress(input),
            Algorithm::Lz4 => lz4::decompress(input),
            Algorithm::Lzw => lzw::decompress(input), 
            Algorithm::Hf => huffman::decompress(input), 
        }
    } else {
        match algorithm {
            Algorithm::Rle => rle::decompress(input),
            Algorithm::Lz77 => lz77::decompress(input),
            Algorithm::Lz4 => lz4::decompress(input),
            Algorithm::Lzw => lzw::decompress(input),
            Algorithm::Hf => huffman::decompress(input), 
        }
    }
}