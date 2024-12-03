pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut count = 1;

    for i in 1..input.len() {
        if input[i] == input[i - 1] {
            count += 1;
        } else {
            compressed.push(count);
            compressed.push(input[i - 1]);
            count = 1;
        }
    }

    // добавление последнего блока
    if !input.is_empty() {
        compressed.push(count);
        compressed.push(input[input.len() - 1]);
    }

    compressed
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::new();

    for chunk in input.chunks(2) {
        if chunk.len() == 2 {
            let count = chunk[0];
            let value = chunk[1];
            decompressed.extend(vec![value; count as usize]);
        }
    }

    decompressed
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress() {
        let input = b"AAAABBBCCDAA";
        let expected = vec![4, b'A', 3, b'B', 2, b'C', 1, b'D', 2, b'A'];
        assert_eq!(compress(input), expected);
    }

    #[test]
    fn test_decompress() {
        let input = vec![4, b'A', 3, b'B', 2, b'C', 1, b'D', 2, b'A'];
        let expected = b"AAAABBBCCDAA".to_vec();
        assert_eq!(decompress(&input), expected);
    }
}

