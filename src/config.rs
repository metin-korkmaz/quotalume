use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_refresh_seconds")]
    pub refresh_seconds: u64,
    #[serde(default)]
    pub openrouter_api_key: Option<String>,
    #[serde(default)]
    pub ollama_api_key: Option<String>,
    #[serde(default)]
    pub ollama_cookie_header: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_seconds: default_refresh_seconds(),
            openrouter_api_key: None,
            ollama_api_key: None,
            ollama_cookie_header: None,
        }
    }
}

fn default_refresh_seconds() -> u64 {
    300
}

impl Config {
    #[must_use]
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("dev", "quotalume", "quotalume")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("~/.config/quotalume"))
    }

    #[must_use]
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Config okunamadı: {}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(Self::default());
        }
        toml::from_str(&raw).context("Config TOML geçersiz")
    }

    pub fn env_override(mut self) -> Self {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.is_empty() {
                self.openrouter_api_key = Some(key);
            }
        }
        if let Ok(key) = std::env::var("OLLAMA_API_KEY") {
            if !key.is_empty() {
                self.ollama_api_key = Some(key);
            }
        }
        if let Ok(cookie) = std::env::var("OLLAMA_COOKIE") {
            if !cookie.is_empty() {
                self.ollama_cookie_header = Some(cookie);
            }
        }
        self
    }
}

pub fn codex_auth_path() -> PathBuf {
    if let Ok(home) = std::env::var("CODEX_HOME") {
        if !home.is_empty() {
            return Path::new(&home).join("auth.json");
        }
    }
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(".codex").join("auth.json"))
        .unwrap_or_else(|| PathBuf::from("~/.codex/auth.json"))
}

pub fn claude_credentials_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        if !dir.is_empty() {
            return Path::new(&dir).join(".credentials.json");
        }
    }
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(".claude").join(".credentials.json"))
        .unwrap_or_else(|| PathBuf::from("~/.claude/.credentials.json"))
}