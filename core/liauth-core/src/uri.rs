use std::collections::HashMap;

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};

use crate::base32;
use crate::model::{Account, Algorithm, TokenKind};
use crate::time::unix_now;
use crate::CoreError;

pub fn parse(uri: &str) -> Result<Account, CoreError> {
    let rest = uri
        .strip_prefix("otpauth://")
        .ok_or_else(|| CoreError::InvalidUri("missing otpauth scheme".into()))?;
    let (kind_str, rest) = rest
        .split_once('/')
        .ok_or_else(|| CoreError::InvalidUri("missing token type".into()))?;
    let (label_raw, query) = match rest.split_once('?') {
        Some((label, query)) => (label, query),
        None => (rest, ""),
    };

    let label = percent_decode_str(label_raw)
        .decode_utf8()
        .map_err(|_| CoreError::InvalidUri("label is not valid utf-8".into()))?
        .into_owned();
    let (label_issuer, account_name) = match label.split_once(':') {
        Some((issuer, name)) => (issuer.trim().to_string(), name.trim().to_string()),
        None => (String::new(), label.trim().to_string()),
    };

    let params = parse_query(query)?;
    let secret_str = params
        .get("secret")
        .ok_or_else(|| CoreError::InvalidUri("missing secret".into()))?;
    let secret = base32::decode(secret_str)?;

    let issuer = params
        .get("issuer")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(label_issuer);

    let algorithm = params
        .get("algorithm")
        .and_then(|a| Algorithm::parse(a))
        .unwrap_or_default();

    let digits = params
        .get("digits")
        .and_then(|d| d.parse::<u32>().ok())
        .filter(|d| (4..=10).contains(d))
        .unwrap_or(6);

    let kind = match kind_str.to_ascii_lowercase().as_str() {
        "totp" => {
            if params
                .get("issuer")
                .map(|i| i.eq_ignore_ascii_case("steam"))
                .unwrap_or(false)
                || params.contains_key("steam")
            {
                TokenKind::Steam
            } else {
                let period = params
                    .get("period")
                    .and_then(|p| p.parse::<u32>().ok())
                    .filter(|p| (5..=300).contains(p))
                    .unwrap_or(30);
                TokenKind::Totp { period }
            }
        }
        "hotp" => {
            let counter = params
                .get("counter")
                .and_then(|c| c.parse::<u64>().ok())
                .unwrap_or(0);
            TokenKind::Hotp { counter }
        }
        "steam" => TokenKind::Steam,
        other => return Err(CoreError::InvalidUri(format!("unsupported token type {other}"))),
    };

    let mut account = Account::new(issuer, account_name, secret, unix_now());
    account.kind = kind;
    account.algorithm = algorithm;
    account.digits = digits;
    Ok(account)
}

pub fn build(account: &Account) -> String {
    let kind = match account.kind {
        TokenKind::Hotp { .. } => "hotp",
        _ => "totp",
    };
    let label = if account.issuer.is_empty() {
        encode_component(&account.name)
    } else {
        format!(
            "{}:{}",
            encode_component(&account.issuer),
            encode_component(&account.name)
        )
    };
    let mut uri = format!(
        "otpauth://{}/{}?secret={}&algorithm={}&digits={}",
        kind,
        label,
        base32::encode(&account.secret.0),
        account.algorithm.name(),
        account.digits
    );
    if !account.issuer.is_empty() {
        uri.push_str("&issuer=");
        uri.push_str(&encode_component(&account.issuer));
    }
    match account.kind {
        TokenKind::Totp { period } => {
            uri.push_str("&period=");
            uri.push_str(&period.to_string());
        }
        TokenKind::Hotp { counter } => {
            uri.push_str("&counter=");
            uri.push_str(&counter.to_string());
        }
        TokenKind::Steam => uri.push_str("&issuer=Steam"),
    }
    uri
}

fn encode_component(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn parse_query(query: &str) -> Result<HashMap<String, String>, CoreError> {
    let mut params = HashMap::new();
    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = percent_decode_str(key)
            .decode_utf8()
            .map_err(|_| CoreError::InvalidUri("invalid query key".into()))?
            .to_ascii_lowercase();
        let value = percent_decode_str(&value.replace('+', " "))
            .decode_utf8()
            .map_err(|_| CoreError::InvalidUri("invalid query value".into()))?
            .into_owned();
        params.insert(key, value);
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_uri() {
        let account = parse(
            "otpauth://totp/GitHub:liwidale?secret=MZXW6YTBOI&issuer=GitHub&algorithm=SHA256&digits=8&period=60",
        )
        .unwrap();
        assert_eq!(account.issuer, "GitHub");
        assert_eq!(account.name, "liwidale");
        assert_eq!(account.algorithm, Algorithm::Sha256);
        assert_eq!(account.digits, 8);
        assert_eq!(account.kind, TokenKind::Totp { period: 60 });
        assert_eq!(account.secret.0, b"foobar");
    }

    #[test]
    fn applies_defaults() {
        let account = parse("otpauth://totp/user%40example.com?secret=MZXW6YTBOI").unwrap();
        assert_eq!(account.name, "user@example.com");
        assert_eq!(account.issuer, "");
        assert_eq!(account.digits, 6);
        assert_eq!(account.kind, TokenKind::Totp { period: 30 });
        assert_eq!(account.algorithm, Algorithm::Sha1);
    }

    #[test]
    fn parses_hotp() {
        let account = parse("otpauth://hotp/Vendor:me?secret=MZXW6YTBOI&counter=7").unwrap();
        assert_eq!(account.kind, TokenKind::Hotp { counter: 7 });
    }

    #[test]
    fn detects_steam() {
        let account = parse("otpauth://totp/Steam:me?secret=MZXW6YTBOI&issuer=Steam").unwrap();
        assert_eq!(account.kind, TokenKind::Steam);
    }

    #[test]
    fn ignores_out_of_range_values() {
        let account =
            parse("otpauth://totp/user?secret=MZXW6YTBOI&digits=99&period=100000&algorithm=MD5").unwrap();
        assert_eq!(account.digits, 6);
        assert_eq!(account.kind, TokenKind::Totp { period: 30 });
        assert_eq!(account.algorithm, Algorithm::Sha1);
    }

    #[test]
    fn roundtrip() {
        let original = parse("otpauth://totp/GitHub:liwidale?secret=MZXW6YTBOI&issuer=GitHub").unwrap();
        let rebuilt = parse(&build(&original)).unwrap();
        assert_eq!(rebuilt.issuer, original.issuer);
        assert_eq!(rebuilt.name, original.name);
        assert_eq!(rebuilt.secret.0, original.secret.0);
        assert_eq!(rebuilt.kind, original.kind);
    }
}
