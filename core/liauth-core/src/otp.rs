use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::model::{Algorithm, TokenKind};

const STEAM_ALPHABET: &[u8; 26] = b"23456789BCDFGHJKMNPQRTVWXY";

fn hmac_digest(algorithm: Algorithm, key: &[u8], message: &[u8]) -> Vec<u8> {
    match algorithm {
        Algorithm::Sha1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("hmac accepts any key length");
            mac.update(message);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("hmac accepts any key length");
            mac.update(message);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key).expect("hmac accepts any key length");
            mac.update(message);
            mac.finalize().into_bytes().to_vec()
        }
    }
}

fn dynamic_truncate(digest: &[u8]) -> u32 {
    let offset = (digest[digest.len() - 1] & 0x0f) as usize;
    (u32::from(digest[offset] & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3])
}

pub fn hotp(secret: &[u8], counter: u64, digits: u32, algorithm: Algorithm) -> String {
    let digest = hmac_digest(algorithm, secret, &counter.to_be_bytes());
    let truncated = dynamic_truncate(&digest);
    let modulo = 10u64.pow(digits.clamp(4, 10));
    let code = u64::from(truncated) % modulo;
    format!("{:0width$}", code, width = digits.clamp(4, 10) as usize)
}

pub fn totp(secret: &[u8], unix_time: i64, period: u32, digits: u32, algorithm: Algorithm) -> String {
    let counter = (unix_time / i64::from(period.max(1))) as u64;
    hotp(secret, counter, digits, algorithm)
}

pub fn steam(secret: &[u8], unix_time: i64) -> String {
    let digest = hmac_digest(Algorithm::Sha1, secret, &((unix_time / 30) as u64).to_be_bytes());
    let mut value = dynamic_truncate(&digest);
    let mut code = String::with_capacity(5);
    for _ in 0..5 {
        code.push(STEAM_ALPHABET[(value % 26) as usize] as char);
        value /= 26;
    }
    code
}

pub fn code_for(secret: &[u8], kind: TokenKind, algorithm: Algorithm, digits: u32, unix_time: i64) -> String {
    match kind {
        TokenKind::Totp { period } => totp(secret, unix_time, period, digits, algorithm),
        TokenKind::Hotp { counter } => hotp(secret, counter, digits, algorithm),
        TokenKind::Steam => steam(secret, unix_time),
    }
}

pub fn seconds_remaining(unix_time: i64, period: u32) -> u32 {
    let period = i64::from(period.max(1));
    (period - unix_time.rem_euclid(period)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC4226_SECRET: &[u8] = b"12345678901234567890";

    #[test]
    fn rfc4226_hotp_vectors() {
        let expected = [
            "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583", "399871",
            "520489",
        ];
        for (counter, code) in expected.iter().enumerate() {
            assert_eq!(hotp(RFC4226_SECRET, counter as u64, 6, Algorithm::Sha1), *code);
        }
    }

    #[test]
    fn rfc6238_totp_vectors() {
        let sha1_secret = b"12345678901234567890";
        let sha256_secret = b"12345678901234567890123456789012";
        let sha512_secret = b"1234567890123456789012345678901234567890123456789012345678901234";
        let cases: [(i64, &str, Algorithm, &[u8]); 9] = [
            (59, "94287082", Algorithm::Sha1, sha1_secret),
            (59, "46119246", Algorithm::Sha256, sha256_secret),
            (59, "90693936", Algorithm::Sha512, sha512_secret),
            (1111111109, "07081804", Algorithm::Sha1, sha1_secret),
            (1111111109, "68084774", Algorithm::Sha256, sha256_secret),
            (1234567890, "89005924", Algorithm::Sha1, sha1_secret),
            (2000000000, "69279037", Algorithm::Sha1, sha1_secret),
            (20000000000, "65353130", Algorithm::Sha1, sha1_secret),
            (20000000000, "47863826", Algorithm::Sha512, sha512_secret),
        ];
        for (time, code, algorithm, secret) in cases {
            assert_eq!(totp(secret, time, 30, 8, algorithm), code);
        }
    }

    #[test]
    fn countdown() {
        assert_eq!(seconds_remaining(0, 30), 30);
        assert_eq!(seconds_remaining(29, 30), 1);
        assert_eq!(seconds_remaining(30, 30), 30);
    }

    #[test]
    fn steam_codes_have_expected_shape() {
        let code = steam(RFC4226_SECRET, 59);
        assert_eq!(code.len(), 5);
        assert!(code.bytes().all(|b| STEAM_ALPHABET.contains(&b)));
    }
}
