use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use liauth_core::time::unix_now;
use liauth_core::{Account, Algorithm, TokenKind};
use percent_encoding::percent_decode_str;

use crate::ImportError;

pub fn import_google_migration(uri: &str) -> Result<Vec<Account>, ImportError> {
    let query = uri
        .split_once('?')
        .map(|(_, q)| q)
        .ok_or_else(|| ImportError::Malformed("missing query".into()))?;
    let data_param = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("data="))
        .ok_or_else(|| ImportError::Malformed("missing data parameter".into()))?;
    let decoded_param = percent_decode_str(&data_param.replace('+', "%2B"))
        .decode_utf8()
        .map_err(|_| ImportError::Malformed("invalid percent encoding".into()))?
        .replace(' ', "+");
    let payload = B64
        .decode(decoded_param.as_bytes())
        .map_err(|_| ImportError::Malformed("invalid base64 payload".into()))?;

    let accounts = parse_payload(&payload)?;
    if accounts.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok(accounts)
}

fn parse_payload(payload: &[u8]) -> Result<Vec<Account>, ImportError> {
    let mut accounts = Vec::new();
    let mut reader = Reader::new(payload);
    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, 2) => {
                let raw = reader.bytes()?;
                accounts.push(parse_parameters(raw)?);
            }
            _ => reader.skip(wire)?,
        }
    }
    Ok(accounts)
}

fn parse_parameters(raw: &[u8]) -> Result<Account, ImportError> {
    let mut secret = Vec::new();
    let mut name = String::new();
    let mut issuer = String::new();
    let mut algorithm = Algorithm::Sha1;
    let mut digits = 6u32;
    let mut kind_hotp = false;
    let mut counter = 0u64;

    let mut reader = Reader::new(raw);
    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, 2) => secret = reader.bytes()?.to_vec(),
            (2, 2) => name = String::from_utf8_lossy(reader.bytes()?).into_owned(),
            (3, 2) => issuer = String::from_utf8_lossy(reader.bytes()?).into_owned(),
            (4, 0) => {
                algorithm = match reader.varint()? {
                    2 => Algorithm::Sha256,
                    3 => Algorithm::Sha512,
                    _ => Algorithm::Sha1,
                }
            }
            (5, 0) => {
                digits = match reader.varint()? {
                    2 => 8,
                    _ => 6,
                }
            }
            (6, 0) => kind_hotp = reader.varint()? == 1,
            (7, 0) => counter = reader.varint()?,
            _ => reader.skip(wire)?,
        }
    }

    if secret.is_empty() {
        return Err(ImportError::Malformed("entry has no secret".into()));
    }

    let (final_issuer, final_name) = match name.split_once(':') {
        Some((prefix, rest)) if issuer.is_empty() || prefix.trim() == issuer => {
            let resolved = if issuer.is_empty() {
                prefix.trim().to_string()
            } else {
                issuer
            };
            (resolved, rest.trim().to_string())
        }
        _ => (issuer, name),
    };

    let mut account = Account::new(final_issuer, final_name, secret, unix_now());
    account.algorithm = algorithm;
    account.digits = digits;
    account.kind = if kind_hotp {
        TokenKind::Hotp { counter }
    } else {
        TokenKind::Totp { period: 30 }
    };
    Ok(account)
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn tag(&mut self) -> Result<Option<(u64, u64)>, ImportError> {
        if self.pos >= self.data.len() {
            return Ok(None);
        }
        let key = self.varint()?;
        Ok(Some((key >> 3, key & 0x7)))
    }

    fn varint(&mut self) -> Result<u64, ImportError> {
        let mut value = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = *self
                .data
                .get(self.pos)
                .ok_or_else(|| ImportError::Malformed("truncated varint".into()))?;
            self.pos += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(ImportError::Malformed("varint overflow".into()));
            }
        }
    }

    fn bytes(&mut self) -> Result<&'a [u8], ImportError> {
        let len = self.varint()? as usize;
        let end = self
            .pos
            .checked_add(len)
            .filter(|end| *end <= self.data.len())
            .ok_or_else(|| ImportError::Malformed("truncated bytes".into()))?;
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn skip(&mut self, wire: u64) -> Result<(), ImportError> {
        match wire {
            0 => {
                self.varint()?;
            }
            1 => self.pos += 8,
            2 => {
                self.bytes()?;
            }
            5 => self.pos += 4,
            _ => return Err(ImportError::Malformed(format!("unsupported wire type {wire}"))),
        }
        if self.pos > self.data.len() {
            return Err(ImportError::Malformed("truncated message".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liauth_core::base32;

    fn varint(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                out.push(byte);
                break;
            }
            out.push(byte | 0x80);
        }
    }

    fn field_bytes(field: u64, data: &[u8], out: &mut Vec<u8>) {
        varint(field << 3 | 2, out);
        varint(data.len() as u64, out);
        out.extend_from_slice(data);
    }

    fn field_varint(field: u64, value: u64, out: &mut Vec<u8>) {
        varint(field << 3, out);
        varint(value, out);
    }

    fn sample_uri() -> String {
        let mut entry = Vec::new();
        field_bytes(1, b"12345678901234567890", &mut entry);
        field_bytes(2, b"GitHub:liwidale", &mut entry);
        field_bytes(3, b"GitHub", &mut entry);
        field_varint(4, 1, &mut entry);
        field_varint(5, 1, &mut entry);
        field_varint(6, 2, &mut entry);

        let mut payload = Vec::new();
        field_bytes(1, &entry, &mut payload);
        field_varint(2, 1, &mut payload);
        field_varint(3, 1, &mut payload);

        let encoded: String = base64::engine::general_purpose::STANDARD.encode(&payload);
        let escaped: String =
            percent_encoding::utf8_percent_encode(&encoded, percent_encoding::NON_ALPHANUMERIC).to_string();
        format!("otpauth-migration://offline?data={escaped}")
    }

    #[test]
    fn parses_migration_payload() {
        let accounts = import_google_migration(&sample_uri()).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].issuer, "GitHub");
        assert_eq!(accounts[0].name, "liwidale");
        assert_eq!(accounts[0].digits, 6);
        assert_eq!(
            base32::encode(&accounts[0].secret.0),
            base32::encode(b"12345678901234567890")
        );
        assert!(matches!(
            accounts[0].kind,
            liauth_core::TokenKind::Totp { period: 30 }
        ));
    }

    #[test]
    fn rejects_garbage() {
        assert!(import_google_migration("otpauth-migration://offline?data=!!!").is_err());
        assert!(import_google_migration("otpauth-migration://offline").is_err());
    }
}
