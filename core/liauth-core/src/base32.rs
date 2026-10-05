use crate::CoreError;

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(5) * 8);
    for chunk in data.chunks(5) {
        let mut buf = [0u8; 5];
        buf[..chunk.len()].copy_from_slice(chunk);
        let bits = u64::from(buf[0]) << 32
            | u64::from(buf[1]) << 24
            | u64::from(buf[2]) << 16
            | u64::from(buf[3]) << 8
            | u64::from(buf[4]);
        let chars = chunk.len() * 8 / 5 + 1;
        for i in 0..8 {
            if i < chars {
                let idx = ((bits >> (35 - i * 5)) & 0x1f) as usize;
                out.push(ALPHABET[idx] as char);
            }
        }
    }
    out
}

pub fn decode(input: &str) -> Result<Vec<u8>, CoreError> {
    let normalized: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
        .map(|c| c.to_ascii_uppercase())
        .map(|c| {
            if c == '0' {
                'O'
            } else if c == '1' {
                'L'
            } else {
                c
            }
        })
        .collect();
    if normalized.is_empty() {
        return Err(CoreError::InvalidBase32);
    }
    let mut bits: u32 = 0;
    let mut bit_count: u32 = 0;
    let mut out = Vec::with_capacity(normalized.len() * 5 / 8);
    for c in normalized.bytes() {
        let value = ALPHABET
            .iter()
            .position(|a| *a == c)
            .ok_or(CoreError::InvalidBase32)? as u32;
        bits = (bits << 5) | value;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            out.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }
    Ok(out)
}

pub fn is_plausible(input: &str) -> bool {
    let stripped: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    stripped.len() >= 8 && stripped.chars().all(|c| c.is_ascii_alphanumeric() || c == '=')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = b"12345678901234567890";
        let encoded = encode(data);
        assert_eq!(decode(&encoded).unwrap(), data);
    }

    #[test]
    fn rfc4648_vectors() {
        assert_eq!(encode(b"foobar"), "MZXW6YTBOI");
        assert_eq!(decode("MZXW6YTBOI======").unwrap(), b"foobar");
        assert_eq!(decode("mzxw 6ytb oi").unwrap(), b"foobar");
    }

    #[test]
    fn rejects_invalid() {
        assert!(decode("!!!").is_err());
        assert!(decode("").is_err());
    }
}
