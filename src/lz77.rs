const WINDOW_SIZE: usize = 4096;
const LOOKAHEAD_BUFFER_SIZE: usize = 18;

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let mut match_length = 0;
        let mut match_distance = 0;

        let start = if i >= WINDOW_SIZE { i - WINDOW_SIZE } else { 0 };

        for j in start..i {
            let mut k = 0;
            while k < LOOKAHEAD_BUFFER_SIZE && i + k < input.len() && input[j + k] == input[i + k] {
                k += 1;
            }
            if k > match_length {
                match_length = k;
                match_distance = i - j;
            }
        }

        if match_length >= 3 {
            compressed.push(0);
            compressed.push((match_distance >> 8) as u8);
            compressed.push((match_distance & 0xFF) as u8);
            compressed.push(match_length as u8);
            i += match_length;
        } else {
            compressed.push(1);
            compressed.push(input[i]);
            i += 1;
        }
    }

    compressed
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == 0 {
            let distance = ((input[i + 1] as usize) << 8) | (input[i + 2] as usize);
            let length = input[i + 3] as usize;
            let start = decompressed.len() - distance;
            for j in 0..length {
                decompressed.push(decompressed[start + j]);
            }
            i += 4;
        } else {
            decompressed.push(input[i + 1]);
            i += 2;
        }
    }

    decompressed
}