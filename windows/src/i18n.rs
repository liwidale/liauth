use std::collections::HashMap;
use std::path::PathBuf;

const EN: &str = include_str!("../../localization/en.json");
const RU: &str = include_str!("../../localization/ru.json");
const DE: &str = include_str!("../../localization/de.json");
const ES: &str = include_str!("../../localization/es.json");
const FR: &str = include_str!("../../localization/fr.json");
const ZH: &str = include_str!("../../localization/zh.json");

pub struct Localization {
    languages: Vec<Language>,
    active: usize,
    fallback: HashMap<String, String>,
}

pub struct Language {
    pub code: String,
    pub name: String,
    strings: HashMap<String, String>,
}

impl Localization {
    pub fn load(extra_dir: Option<PathBuf>, preferred: Option<&str>) -> Self {
        let mut languages = Vec::new();
        for (code, raw) in [
            ("en", EN),
            ("ru", RU),
            ("de", DE),
            ("es", ES),
            ("fr", FR),
            ("zh", ZH),
        ] {
            if let Some(language) = parse_language(code, raw) {
                languages.push(language);
            }
        }
        if let Some(dir) = extra_dir {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let Some(code) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let Ok(raw) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    if let Some(language) = parse_language(code, &raw) {
                        if let Some(existing) = languages.iter_mut().find(|l| l.code == language.code) {
                            *existing = language;
                        } else {
                            languages.push(language);
                        }
                    }
                }
            }
        }

        let fallback = languages
            .iter()
            .find(|l| l.code == "en")
            .map(|l| l.strings.clone())
            .unwrap_or_default();

        let requested = preferred.map(str::to_string).unwrap_or_else(system_language);
        let active = languages
            .iter()
            .position(|l| l.code == requested)
            .or_else(|| {
                let prefix = requested.split(['-', '_']).next().unwrap_or("");
                languages.iter().position(|l| l.code == prefix)
            })
            .unwrap_or(0);

        Self {
            languages,
            active,
            fallback,
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.languages
            .get(self.active)
            .and_then(|l| l.strings.get(key))
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut value = self.t(key);
        for (name, replacement) in args {
            value = value.replace(&format!("{{{name}}}"), replacement);
        }
        value
    }

    pub fn languages(&self) -> impl Iterator<Item = (&str, &str)> {
        self.languages.iter().map(|l| (l.code.as_str(), l.name.as_str()))
    }

    pub fn active_code(&self) -> &str {
        self.languages
            .get(self.active)
            .map(|l| l.code.as_str())
            .unwrap_or("en")
    }

    pub fn set_language(&mut self, code: &str) {
        if let Some(index) = self.languages.iter().position(|l| l.code == code) {
            self.active = index;
        }
    }
}

fn parse_language(code: &str, raw: &str) -> Option<Language> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let object = value.as_object()?;
    let mut strings = HashMap::new();
    for (key, entry) in object {
        if let Some(text) = entry.as_str() {
            strings.insert(key.clone(), text.to_string());
        }
    }
    let name = strings
        .get("language.name")
        .cloned()
        .unwrap_or_else(|| code.to_uppercase());
    Some(Language {
        code: code.to_string(),
        name,
        strings,
    })
}

fn system_language() -> String {
    std::env::var("LANG")
        .ok()
        .or_else(sys_locale)
        .map(|l| l.split(['-', '_', '.']).next().unwrap_or("en").to_lowercase())
        .unwrap_or_else(|| "en".to_string())
}

#[cfg(windows)]
fn sys_locale() -> Option<String> {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    let mut buffer = [0u16; 85];
    let length = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if length > 1 {
        Some(String::from_utf16_lossy(&buffer[..length as usize - 1]))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn sys_locale() -> Option<String> {
    None
}
