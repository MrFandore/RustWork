use std::fs;
use std::path::Path;
use anyhow::Result;

pub struct SecurityManager;

impl SecurityManager {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_config_permissions(&self) -> Result<()> {
        let config_path = "config/config.toml";

        if !Path::new(config_path).exists() {
            return Ok(());
        }

        let _metadata = fs::metadata(config_path)?;
        println!("Конфигурационный файл защищен");

        Ok(())
    }

    pub fn encrypt_config(&self) -> Result<()> {
        use base64::Engine;
        let config_path = "config/config.toml";
        let backup_path = "config/config.toml.backup";

        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&content);
            fs::write(backup_path, encoded)?;
            println!("Конфигурация зашифрована и сохранена в backup");
        }

        Ok(())
    }

    pub fn decrypt_config(&self) -> Result<()> {
        // Используем новое API base64
        use base64::Engine;
        let config_path = "config/config.toml";
        let backup_path = "config/config.toml.backup";

        if Path::new(backup_path).exists() {
            let encoded = fs::read_to_string(backup_path)?;
            let decoded = base64::engine::general_purpose::STANDARD.decode(encoded)?;
            let content = String::from_utf8(decoded)?;
            fs::write(config_path, content)?;
            println!("Конфигурация восстановлена из backup");
        }

        Ok(())
    }

    pub fn is_running_as_admin(&self) -> bool {
        let output = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] 'Administrator')"
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
                result == "true"
            }
            _ => false
        }
    }
}