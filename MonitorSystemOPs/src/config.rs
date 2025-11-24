use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

const DEFAULT_CONFIG: &str = r#"
[monitoring]
interval_seconds = 5

[storage]
max_records = 1000

[web]
host = "127.0.0.1"
port = 8080
"#;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub monitoring: MonitoringConfig,
    pub storage: StorageConfig,
    pub web: WebConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub max_records: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config/config.toml";

        if !std::path::Path::new(config_path).exists() {
            Self::generate_default()?;
            println!("Создан файл конфигурации по умолчанию: {}", config_path);
        }

        let config_content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }

    pub fn generate_default() -> Result<()> {
        let config_dir = "config";
        if !std::path::Path::new(config_dir).exists() {
            fs::create_dir_all(config_dir)?;
        }

        fs::write("config/config.toml", DEFAULT_CONFIG.trim())?;
        Ok(())
    }
}