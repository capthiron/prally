use figment::{Figment, providers::{Format, Toml, Env}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{PrallyError, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub core: CoreConfig,
    pub github: Option<GitHubConfig>,
    pub gitlab: Option<GitLabConfig>,
    pub jira: Option<JiraConfig>,
    pub llm: LlmConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreConfig {
    pub default_branch: String,
    pub auto_fetch: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubConfig {
    pub base_url: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabConfig {
    pub base_url: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraConfig {
    pub base_url: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig {
                default_branch: "main".to_string(),
                auto_fetch: true,
            },
            github: None,
            gitlab: None,
            jira: None,
            llm: LlmConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                max_tokens: 2000,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_dir()?.join("config.toml");

        let figment = Figment::new()
            .merge(Toml::file(config_path))
            .merge(Env::prefixed("PRALLY_"));

        figment
            .extract()
            .or_else(|_| Ok(Self::default()))
            .map_err(|e: figment::Error| PrallyError::Config(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| PrallyError::Config(e.to_string()))?;

        std::fs::write(config_path, toml_string)?;
        Ok(())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "core.default_branch" => self.core.default_branch = value.to_string(),
            "core.auto_fetch" => self.core.auto_fetch = value.parse().map_err(|_|
                PrallyError::Config("Invalid boolean value".to_string()))?,
            "llm.provider" => self.llm.provider = value.to_string(),
            "llm.model" => self.llm.model = value.to_string(),
            "llm.max_tokens" => self.llm.max_tokens = value.parse().map_err(|_|
                PrallyError::Config("Invalid number value".to_string()))?,
            _ => return Err(PrallyError::Config(format!("Unknown config key: {}", key))),
        }
        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Result<String> {
        let value = match key {
            "core.default_branch" => &self.core.default_branch,
            "core.auto_fetch" => return Ok(self.core.auto_fetch.to_string()),
            "llm.provider" => &self.llm.provider,
            "llm.model" => &self.llm.model,
            "llm.max_tokens" => return Ok(self.llm.max_tokens.to_string()),
            _ => return Err(PrallyError::Config(format!("Unknown config key: {}", key))),
        };
        Ok(value.clone())
    }

    fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| PrallyError::Config("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home).join(".config").join("prally"))
    }
}
