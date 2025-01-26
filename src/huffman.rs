use std::collections::{BinaryHeap, HashMap};

#[derive(Eq, PartialEq)]
struct Node {
    freq: usize,
    byte: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by frequency first
        let freq_order = other.freq.cmp(&self.freq);
        if freq_order == std::cmp::Ordering::Equal {
            // Tiebreak by byte value to ensure consistent ordering
            let self_byte = self.byte.unwrap_or(0);
            let other_byte = other.byte.unwrap_or(0);
            return self_byte.cmp(&other_byte);
        }
        freq_order
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn build_huffman_tree(freq_map: &HashMap<u8, usize>) -> Option<Box<Node>> {
    let mut freq_vec: Vec<(u8, usize)> = freq_map.iter().map(|(&b, &f)| (b, f)).collect();
    // Sort by byte to ensure consistent insertion order
    freq_vec.sort_by_key(|(b, _)| *b);

    let mut heap = BinaryHeap::new();
    for (b, f) in freq_vec {
        heap.push(Box::new(Node {
            freq: f,
            byte: Some(b),
            left: None,
            right: None,
        }));
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();
        heap.push(Box::new(Node {
            freq: left.freq + right.freq,
            byte: None,
            left: Some(left),
            right: Some(right),
        }));
    }
    heap.pop()
}

fn build_codes(node: &Option<Box<Node>>, prefix: Vec<bool>, codes: &mut HashMap<u8, Vec<bool>>) {
    if let Some(n) = node {
        if let Some(b) = n.byte {
            codes.insert(b, prefix);
        } else {
            let mut left_prefix = prefix.clone();
            left_prefix.push(false);
            build_codes(&n.left, left_prefix, codes);

            let mut right_prefix = prefix;
            right_prefix.push(true);
            build_codes(&n.right, right_prefix, codes);
        }
    }
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![];
    }
    let original_len = input.len() as u32;
    let mut freq_map = HashMap::new();
    for &b in input {
        *freq_map.entry(b).or_insert(0) += 1;
    }
    let root = build_huffman_tree(&freq_map);
    let mut codes = HashMap::new();
    build_codes(&root, vec![], &mut codes);

    let mut header = Vec::new();
    header.extend_from_slice(&original_len.to_be_bytes());

    // Записываем размер словаря (u16 вместо u8)
    let dict_len = freq_map.len() as u16;
    header.extend_from_slice(&dict_len.to_be_bytes());

    for (b, f) in freq_map {
        header.push(b);
        header.extend_from_slice(&(f as u32).to_be_bytes());
    }

    // Формируем биты согласно кодам Хаффмана
    let mut bits = Vec::new();
    for &b in input {
        if let Some(code) = codes.get(&b) {
            bits.extend_from_slice(code);
        }
    }

    // Упаковываем биты в байты
    let mut packed = Vec::new();
    let mut byte = 0u8;
    let mut bit_index = 0;
    for bit in bits {
        byte <<= 1;
        if bit {
            byte |= 1;
        }
        bit_index += 1;
        if bit_index == 8 {
            packed.push(byte);
            byte = 0;
            bit_index = 0;
        }
    }
    if bit_index != 0 {
        packed.push(byte << (8 - bit_index));
    }

    // Добавляем длину упакованных данных (4 байта) и сами данные
    let mut compressed = header;
    compressed.extend_from_slice(&(packed.len() as u32).to_be_bytes());
    compressed.extend_from_slice(&packed);
    compressed
}

// ...existing code...
pub fn decompress(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![];
    }

    let mut idx = 0;
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&input[idx..idx + 4]);
    idx += 4;
    let original_len = u32::from_be_bytes(buf) as usize;

    // Читаем размер словаря (2 байта)
    let mut buf2 = [0u8; 2];
    buf2.copy_from_slice(&input[idx..idx + 2]);
    idx += 2;
    let dict_len = u16::from_be_bytes(buf2) as usize;

    let mut freq_map = HashMap::new();
    for _ in 0..dict_len {
        let b = input[idx];
        idx += 1;
        let mut freq_buf = [0u8; 4];
        freq_buf.copy_from_slice(&input[idx..idx + 4]);
        idx += 4;
        let f = u32::from_be_bytes(freq_buf) as usize;
        freq_map.insert(b, f);
    }

    // Читаем длину упакованных данных
    let data_len = {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&input[idx..idx + 4]);
        idx += 4;
        u32::from_be_bytes(buf) as usize
    };
    // Извлекаем упакованные биты
    let packed = &input[idx..(idx + data_len)];
    let root = build_huffman_tree(&freq_map);

    let mut bits = Vec::new();
    for &p in packed {
        for i in 0..8 {
            bits.push((p & (1 << (7 - i))) != 0);
        }
    }

    let mut node = &root;
    let mut decompressed = Vec::with_capacity(original_len);
    // Распаковываем, пока не достигнем исходной длины
    for bit in bits {
        if let Some(n) = node {
            if bit {
                node = &n.right;
            } else {
                node = &n.left;
            }
            if let Some(real_node) = node {
                if let Some(b) = real_node.byte {
                    decompressed.push(b);
                    node = &root;
                    if decompressed.len() == original_len {
                        break;
                    }
                }
            }
        }
    }
    decompressed
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let input = b"AAAABBBCCDAA";
        let compressed = compress(input);
        let decompressed = decompress(&compressed);
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_empty_input() {
        let input: &[u8] = &[];
        let compressed = compress(input);
        let decompressed = decompress(&compressed);
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_single_byte() {
        let input = b"A";
        let compressed = compress(input);
        let decompressed = decompress(&compressed);
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_varied_input() {
        let input = b"The quick brown fox jumps over the lazy dog";
        let compressed = compress(input);
        let decompressed = decompress(&compressed);
        assert_eq!(decompressed, input);
    }
}