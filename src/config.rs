use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    
    #[serde(default)]
    pub logging: LoggingConfig,
    
    #[serde(default)]
    pub download: DownloadConfig,
    
    #[serde(default)]
    pub monitor: MonitorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    #[serde(default = "default_download_dir")]
    pub dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(default = "default_backoff_initial")]
    pub backoff_initial_ms: u64,
    
    #[serde(default = "default_backoff_max")]
    pub backoff_max_ms: u64,
    
    #[serde(default = "default_jitter")]
    pub jitter: bool,
    
    #[serde(default = "default_logs_follow")]
    pub logs_follow_default: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_download_dir() -> PathBuf {
    PathBuf::from(".")
}

fn default_backoff_initial() -> u64 {
    1000
}

fn default_backoff_max() -> u64 {
    30000
}

fn default_jitter() -> bool {
    true
}

fn default_logs_follow() -> bool {
    false
}

#[derive(Debug, Clone)]
pub struct CliOverrides {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl Config {
    pub fn load(config_file: Option<PathBuf>, cli_overrides: &CliOverrides) -> Result<Self> {
        let mut config = if let Some(config_path) = config_file {
            Self::from_file(&config_path)?
        } else if Path::new("appian-config.toml").exists() {
            Self::from_file(Path::new("appian-config.toml"))?
        } else {
            Self::from_env()?
        };

        config.apply_cli_overrides(cli_overrides);
        config.validate()?;
        
        debug!("Loaded configuration: {:?}", config);
        Ok(config)
    }

    fn from_file(path: &Path) -> Result<Self> {
        info!("Loading configuration from: {}", path.display());
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }

    fn from_env() -> Result<Self> {
        debug!("Loading configuration from environment variables");
        
        let base_url = std::env::var("APPIAN_BASE_URL")
            .unwrap_or_else(|_| "https://mysite.appiancloud.com".to_string());
        
        let api_key = std::env::var("APPIAN_API_KEY")
            .unwrap_or_default();
        
        let timeout_seconds = std::env::var("APPIAN_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        Ok(Config {
            base_url,
            api_key,
            timeout_seconds,
            logging: LoggingConfig::default(),
            download: DownloadConfig::default(),
            monitor: MonitorConfig::default(),
        })
    }

    fn apply_cli_overrides(&mut self, cli: &CliOverrides) {
        if let Some(base_url) = &cli.base_url {
            self.base_url = base_url.clone();
        }
        
        if let Some(api_key) = &cli.api_key {
            self.api_key = api_key.clone();
        }
    }

    fn validate(&self) -> Result<()> {
        if self.base_url.is_empty() {
            anyhow::bail!("base_url cannot be empty");
        }

        if self.api_key.is_empty() {
            anyhow::bail!("api_key cannot be empty");
        }

        if self.timeout_seconds == 0 {
            anyhow::bail!("timeout_seconds must be greater than 0");
        }

        Ok(())
    }

    pub fn get_api_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::from_env().unwrap();
        assert_eq!(config.timeout_seconds, 300);
        assert_eq!(config.logging.level, "info");
        assert!(!config.logging.json);
    }

    #[test]
    fn test_api_url_construction() {
        let config = Config {
            base_url: "https://example.com".to_string(),
            api_key: "test".to_string(),
            timeout_seconds: 300,
            logging: LoggingConfig::default(),
            download: DownloadConfig::default(),
            monitor: MonitorConfig::default(),
        };

        assert_eq!(config.get_api_url("api/v1/test"), "https://example.com/api/v1/test");
        assert_eq!(config.get_api_url("/api/v1/test"), "https://example.com/api/v1/test");
        assert_eq!(config.get_api_url("test"), "https://example.com/test");
    }
}
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json: false,
        }
    }
}
impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            dir: default_download_dir(),
        }
    }
}
impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            backoff_initial_ms: default_backoff_initial(),
            backoff_max_ms: default_backoff_max(),
            jitter: default_jitter(),
            logs_follow_default: default_logs_follow(),
        }
    }
}