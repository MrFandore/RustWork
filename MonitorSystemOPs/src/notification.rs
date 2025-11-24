use std::process::Command;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Notification {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: String,
}

pub struct NotificationSystem;

impl NotificationSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn send_start_notification(&self) {
        let notification = Notification {
            timestamp: Utc::now().to_rfc3339(),
            level: "INFO".to_string(),
            message: "Служба мониторинга запущена".to_string(),
            service: "MonitorSystemOPs".to_string(),
        };
        self.log_notification(&notification);
        self.show_system_notification("MonitorSystemOPs", "Служба мониторинга запущена");
    }

    pub fn send_stop_notification(&self) {
        let notification = Notification {
            timestamp: Utc::now().to_rfc3339(),
            level: "INFO".to_string(),
            message: "Служба мониторинга остановлена".to_string(),
            service: "MonitorSystemOPs".to_string(),
        };
        self.log_notification(&notification);
        self.show_system_notification("MonitorSystemOPs", "Служба мониторинга остановлена");
    }

    pub fn send_error_notification(&self, error: &str) {
        let notification = Notification {
            timestamp: Utc::now().to_rfc3339(),
            level: "ERROR".to_string(),
            message: format!("Ошибка: {}", error),
            service: "MonitorSystemOPs".to_string(),
        };
        self.log_notification(&notification);
        self.show_system_notification("MonitorSystemOPs - Ошибка", error);
    }

    pub fn send_anomaly_notification(&self, anomalies: &[String]) {
        if anomalies.is_empty() {
            return;
        }

        let message = anomalies.join("; ");
        let notification = Notification {
            timestamp: Utc::now().to_rfc3339(),
            level: "WARNING".to_string(),
            message: format!("Обнаружены аномалии: {}", message),
            service: "MonitorSystemOPs".to_string(),
        };
        self.log_notification(&notification);
        self.show_system_notification("MonitorSystemOPs - Предупреждение", &message);
    }

    fn log_notification(&self, notification: &Notification) {
        // Записываем уведомление в лог-файл
        if let Ok(log_entry) = serde_json::to_string(notification) {
            let _ = std::fs::create_dir_all("logs");
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/notifications.log")
            {
                use std::io::Write;
                let _ = writeln!(file, "{}", log_entry);
            }
        }

        // Также выводим в консоль
        println!("[{}] {}: {}", notification.level, notification.timestamp, notification.message);
    }

    fn show_system_notification(&self, title: &str, message: &str) {
        // Используем PowerShell для показа системных уведомлений
        let _ = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{}', '{}')", message, title)
            ])
            .output();
    }
}