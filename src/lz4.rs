//! Модуль для сжатия и распаковки данных с использованием алгоритма LZ4.
//!
//! Этот модуль предоставляет функции для сжатия и распаковки данных с использованием алгоритма LZ4. 
//! Алгоритм LZ4 используется для быстрого сжатия и разжатия данных.


/// Сжимает входные данные с использованием алгоритма LZ4.
///
/// # Аргументы
///
/// * `input` - Срез байтов, которые требуется сжать.
///
/// # Возвращает
///
/// Вектор байтов, представляющий сжатые данные.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut hash_table = vec![-1isize; 65536];
    let input_len = input.len() as isize;
    let mut i = 0isize;

    while i < input_len {
        let mut match_length = 0;
        let mut match_distance = 0;

        if i + 4 <= input_len {
            let sequence = &input[i as usize..(i + 4) as usize];
            let hash = ((sequence[0] as u32) << 8 | sequence[1] as u32) % 65536;
            let ref_pos = hash_table[hash as usize];
            hash_table[hash as usize] = i;

            if ref_pos != -1 && i - ref_pos <= 65535 {
                let mut ref_i = ref_pos as usize;
                let mut s = i as usize;
                let max_length = 255.min(input.len() - s);

                while s < input.len()
                    && input[s] == input[ref_i]
                    && match_length < max_length
                {
                    s += 1;
                    ref_i += 1;
                    match_length += 1;
                }

                match_distance = (i - ref_pos) as usize;
            }
        }

        if match_length >= 4 {
            output.push(0);
            output.extend_from_slice(&(match_distance as u16).to_le_bytes());
            output.push(match_length as u8);
            i += match_length as isize;
        } else {
            output.push(1);
            output.push(input[i as usize]);
            i += 1;
        }
    }

    output
}

/// Распаковывает сжатые данные, используя алгоритм LZ4.
///
/// # Аргументы
///
/// * `input` - Срез байтов, которые требуется распаковать.
///
/// # Возвращает
///
/// Вектор байтов, представляющий распакованные данные.
pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == 0 {
            // проверка, что достаточно данных для чтения offset и length
            if i + 3 >= input.len() {
                eprintln!("Error: Unexpected end of input while reading match block.");
                return Vec::new();
            }

            let offset = u16::from_le_bytes([input[i + 1], input[i + 2]]) as usize;
            let length = input[i + 3] as usize;

            if offset == 0 || offset > output.len() {
                eprintln!("Error: Invalid offset ({}) at position {}.", offset, i);
                return Vec::new();
            }

            let mut start = output.len() - offset;

            
            for _ in 0..length {
                if start >= output.len() {
                    eprintln!("Error: Out of bounds access during decompression.");
                    return Vec::new();
                }
                let byte = output[start];
                output.push(byte);
                start += 1; 
            }

            i += 4;
        } else if input[i] == 1 {
            // Проверка, что достаточно данных для чтения литерала
            if i + 1 >= input.len() {
                eprintln!("Error: Unexpected end of input while reading literal.");
                return Vec::new();
            }

            output.push(input[i + 1]);
            i += 2;
        } else {
            eprintln!("Error: Invalid marker ({}) at position {}.", input[i], i);
            return Vec::new();
        }
    }

    output
}