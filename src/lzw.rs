//! Модуль для сжатия и распаковки данных с использованием алгоритма LZW.
//!
//! Этот модуль предоставляет функции для сжатия и распаковки данных с использованием алгоритма LZW. 
//! Алгоритм LZW используется для сжатия данных путем замены повторяющихся последовательностей кодами из словаря.

use std::collections::HashMap;
use std::io::{self, Write, Read};

/// Структура для записи битов в поток.
struct BitWriter<W: Write> {
    writer: W,
    current_byte: u8,
    bit_position: u8, 
}

impl<W: Write> BitWriter<W> {
     /// Создает новый BitWriter.
    ///
    /// # Аргументы
    ///
    /// * `writer` - Поток для записи.
    ///
    /// # Возвращает
    ///
    /// Новый экземпляр BitWriter.
    fn new(writer: W) -> Self {
        BitWriter {
            writer,
            current_byte: 0,
            bit_position: 0,
        }
    }
/// Записывает заданное количество битов в поток.
    ///
    /// # Аргументы
    ///
    /// * `bits` - Биты для записи.
    /// * `num_bits` - Количество битов для записи.
    ///
    /// # Возвращает
    ///
    /// Результат операции записи.
    fn write_bits(&mut self, bits: u16, num_bits: u8) -> io::Result<()> {
        let mut bits = bits;
        for _ in 0..num_bits {
            let bit = (bits >> (num_bits - 1)) & 1;
            self.current_byte = (self.current_byte << 1) | bit as u8;
            self.bit_position += 1;
            if self.bit_position == 8 {
                self.writer.write_all(&[self.current_byte])?;
                self.current_byte = 0;
                self.bit_position = 0;
            }
            bits <<= 1;
        }
        Ok(())
    }
    
    /// Завершает запись и очищает буфер.
    ///
    /// # Возвращает
    ///
    /// Результат операции записи.
    fn flush(&mut self) -> io::Result<()> {
        if self.bit_position > 0 {
            self.current_byte <<= 8 - self.bit_position;
            self.writer.write_all(&[self.current_byte])?;
            self.current_byte = 0;
            self.bit_position = 0;
        }
        self.writer.flush()
    }
}


/// Структура для чтения битов из потока.
struct BitReader<R: Read> {

    reader: R,
    current_byte: u8,
    bit_position: u8, 
    eof: bool,
}

impl<R: Read> BitReader<R> {
         /// Создает новый BitReader.
    ///
    /// # Аргументы
    ///
    /// * `reader` - Поток для чтения.
    ///
    /// # Возвращает
    ///
    /// Новый экземпляр BitReader.
    fn new(reader: R) -> Self {
        BitReader {
            reader,
            current_byte: 0,
            bit_position: 8, // 8 для чтения первого байта
            eof: false,
        }
    }
    /// Читает заданное количество битов из потока.
    ///
    /// # Аргументы
    ///
    /// * `num_bits` - Количество битов для чтения.
    ///
    /// # Возвращает
    ///
    /// Опциональное значение, содержащее прочитанные биты.

    fn read_bits(&mut self, num_bits: u8) -> io::Result<Option<u16>> {
        let mut result: u16 = 0;
        for _ in 0..num_bits {
            if self.bit_position == 8 {
                let mut buf = [0];
                match self.reader.read_exact(&mut buf) {
                    Ok(_) => self.current_byte = buf[0],
                    Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        self.eof = true;
                        return Ok(None);
                    }
                    Err(e) => return Err(e),
                }
                self.bit_position = 0;
            }
            let bit = (self.current_byte >> (7 - self.bit_position)) & 1;
            result = (result << 1) | bit as u16;
            self.bit_position += 1;
        }
        Ok(Some(result))
    }
}

/// Максимальный размер словаря 
const MAX_DICT_SIZE: u16 = 4096; 

/// Сжимает входные данные с использованием алгоритма LZW.
///
/// # Аргументы
///
/// * `input` - Срез байтов, которые требуется сжать.
///
/// # Возвращает
///
/// Вектор байтов, представляющий сжатые данные.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut dictionary: HashMap<Vec<u8>, u16> = HashMap::new();
    let mut dict_size: u16 = 256;
    for i in 0..256 {
        dictionary.insert(vec![i as u8], i);
    }

    let mut w: Vec<u8> = Vec::new();
    let mut result: Vec<u8> = Vec::new();
    let mut bit_writer = BitWriter::new(&mut result);

    for &c in input.iter() {
        let mut wc = w.clone();
        wc.push(c);
        if dictionary.contains_key(&wc) {
            w = wc;
        } else {
            if let Some(&code) = dictionary.get(&w) {
                bit_writer.write_bits(code, 12).expect("Failed to write bits");
            }
            if dict_size < MAX_DICT_SIZE {
                dictionary.insert(wc, dict_size);
                dict_size += 1;
            }
            w = vec![c];
        }
    }

    if !w.is_empty() {
        if let Some(&code) = dictionary.get(&w) {
            bit_writer.write_bits(code, 12).expect("Failed to write bits");
        }
    }

    bit_writer.flush().expect("Failed to flush bits");
    result
}

/// Распаковывает сжатые данные, используя алгоритм LZW.
///
/// # Аргументы
///
/// * `input` - Срез байтов, которые требуется распаковать.
///
/// # Возвращает
///
/// Вектор байтов, представляющий распакованные данные.
pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut bit_reader = BitReader::new(&input[..]);
    let mut codes: Vec<u16> = Vec::new();

    while let Some(code) = bit_reader.read_bits(12).expect("Failed to read bits") {
        codes.push(code);
    }

    let mut dictionary: HashMap<u16, Vec<u8>> = HashMap::new();
    let mut dict_size: u16 = 256;
    for i in 0..256 {
        dictionary.insert(i, vec![i as u8]);
    }

    let mut result: Vec<u8> = Vec::new();
    let mut w = match codes.get(0) {
        Some(&k) => {
            let entry = dictionary.get(&k).cloned().unwrap_or_else(Vec::new);
            result.extend(&entry);
            entry
        },
        None => return result,
    };

    for &k in codes.iter().skip(1) {
        let entry = if let Some(e) = dictionary.get(&k) {
            e.clone()
        } else if k == dict_size {
            let mut e = w.clone();
            e.push(w[0]);
            e
        } else {
            eprintln!("Error: Invalid LZW code {}", k);
            return Vec::new();
        };
        result.extend(&entry);

        if dict_size < MAX_DICT_SIZE {
            let mut new_entry = w.clone();
            new_entry.push(entry[0]);
            dictionary.insert(dict_size, new_entry);
            dict_size += 1;
        }

        w = entry;
    }

    result
}

