use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Algorithm {
    #[default]
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_uppercase().as_str() {
            "SHA1" => Some(Self::Sha1),
            "SHA256" => Some(Self::Sha256),
            "SHA512" => Some(Self::Sha512),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha512 => "SHA512",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TokenKind {
    Totp { period: u32 },
    Hotp { counter: u64 },
    Steam,
}

impl Default for TokenKind {
    fn default() -> Self {
        Self::Totp { period: 30 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Secret(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub issuer: String,
    pub name: String,
    pub secret: Secret,
    pub kind: TokenKind,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub category_id: Option<Uuid>,
    pub pinned: bool,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub recovery_codes: Vec<String>,
    #[serde(default)]
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Account {
    pub fn new(issuer: String, name: String, secret: Vec<u8>, now: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            issuer,
            name,
            secret: Secret(secret),
            kind: TokenKind::default(),
            algorithm: Algorithm::default(),
            digits: 6,
            category_id: None,
            pinned: false,
            notes: String::new(),
            recovery_codes: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn display_title(&self) -> &str {
        if self.issuer.is_empty() {
            &self.name
        } else {
            &self.issuer
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub position: u32,
}

impl Category {
    pub fn new(name: String, position: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            position,
        }
    }
}
