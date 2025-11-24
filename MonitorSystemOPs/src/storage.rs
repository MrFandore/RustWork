use crate::monitor::SystemMetrics;
use serde_json;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use anyhow::Result;

const DATA_FILE: &str = "data/metrics.json";

pub struct Storage;

impl Storage {
    pub fn new() -> Self {
        // Создаем директорию, если не существует
        let _ = fs::create_dir_all("data");
        Self
    }

    pub fn save_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        let file = File::options()
            .create(true)
            .append(true)
            .open(DATA_FILE)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, metrics)?;
        writeln!(writer)?; // Добавляем новую строку для следующей записи
        Ok(())
    }

    pub fn load_metrics(&self) -> Result<Vec<SystemMetrics>> {
        if !std::path::Path::new(DATA_FILE).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(DATA_FILE)?;
        let mut metrics = Vec::new();
        for line in content.lines() {
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<SystemMetrics>(line) {
                Ok(metric) => metrics.push(metric),
                Err(e) => eprintln!("Ошибка парсинга метрики: {}", e),
            }
        }
        Ok(metrics)
    }

    pub fn cleanup_old_records(&self, max_records: usize) -> Result<()> {
        let mut metrics = self.load_metrics()?;
        if metrics.len() > max_records {
            metrics.drain(0..metrics.len() - max_records);
            let file = File::create(DATA_FILE)?;
            let mut writer = BufWriter::new(file);
            for metric in metrics {
                serde_json::to_writer(&mut writer, &metric)?;
                writeln!(writer)?;
            }
        }
        Ok(())
    }
}