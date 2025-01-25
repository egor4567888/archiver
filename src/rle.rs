pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut i = 0;

    while i < input.len() {
        // Check for run of repeated bytes
        let mut run_len = 1;
        while i + run_len < input.len()
            && input[i + run_len] == input[i]
            && run_len < 127
        {
            run_len += 1;
        }

        if run_len > 1 {
            compressed.push(run_len as u8);
            compressed.push(input[i]);
            i += run_len;
        } else {

            if i + 2 >= input.len() {
                if i + 1 >= input.len() {
                    compressed.push(1);
                    compressed.push(input[i]);
                    i += 1;
                } else
                {
                    compressed.push(128+2);
                compressed.push(input[i]);
                compressed.push(input[i+1]);
                i += 2;
                
            }
            continue;
            }
            // Collect distinct bytes
            let distinct_start = i;
            let mut distinct_count: usize = 2;
            i += 2;
            while i < input.len()
                && !(input[i] == input[i - 1] && input[i] == input[i - 2])
                && distinct_count < 127
            {
                distinct_count += 1;
                i += 1;
            }
            distinct_count = distinct_count.saturating_sub(2);
            compressed.push(128 + distinct_count as u8);
            compressed.extend_from_slice(&input[distinct_start..distinct_start + distinct_count]);
            
                i = i.saturating_sub(2);

        }
    }

    compressed
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let count = input[i];
        i += 1;
        if count <= 127 {
            if i < input.len() {
                let value = input[i];
                i += 1;
                decompressed.extend(std::iter::repeat(value).take(count as usize));
            }
        } else {
            let distinct_count = (count - 128) as usize;
            if i + distinct_count <= input.len() {
                decompressed.extend_from_slice(&input[i..i + distinct_count]);
                i += distinct_count;
            } else {
                break;
            }
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

