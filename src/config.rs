use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_MODEL: &str = "gemini-2.0-flash";

// APIから取得できない場合のフォールバック用
pub const FALLBACK_MODELS: &[&str] = &[
    "gemini-2.0-flash",
    "gemini-2.0-flash-lite",
    "gemini-1.5-flash",
    "gemini-1.5-pro",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputMode {
    #[default]
    Detailed,
    Concise,
}

impl OutputMode {
    pub fn label(&self) -> &'static str {
        match self {
            OutputMode::Detailed => "詳細（言語判定・翻訳・スラング解説・要約）",
            OutputMode::Concise => "簡潔（5行以内で要点のみ）",
        }
    }

    pub fn all() -> &'static [OutputMode] {
        &[OutputMode::Detailed, OutputMode::Concise]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key_code: i32, // Windows VK code
}

impl Default for Hotkey {
    fn default() -> Self {
        // デフォルトは Ctrl+Shift+T (VK_T = 0x54)
        Self {
            ctrl: true,
            alt: false,
            shift: true,
            key_code: 0x54, // VK_T
        }
    }
}

impl Hotkey {
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }

        // キーコードを文字に変換（簡易版）
        let key_name = match self.key_code {
            0x41..=0x5A => {
                // A-Z
                char::from_u32(self.key_code as u32).unwrap_or('?').to_string()
            }
            0x30..=0x39 => {
                // 0-9
                char::from_u32(self.key_code as u32).unwrap_or('?').to_string()
            }
            _ => format!("Key{:X}", self.key_code),
        };

        parts.push(&key_name);
        parts.join("+")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub output_mode: OutputMode,
    #[serde(default)]
    pub hotkey: Hotkey,
}

fn default_model() -> String {
    DEFAULT_MODEL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: DEFAULT_MODEL.to_string(),
            output_mode: OutputMode::default(),
            hotkey: Hotkey::default(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not determine config directory")?;
    path.push("ClipboardTranslator");
    fs::create_dir_all(&path)?;
    path.push("config.json");
    Ok(path)
}

pub fn load_or_create() -> Result<Config> {
    let path = config_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let config = Config::default();
        save(&config)?;
        Ok(config)
    }
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}
