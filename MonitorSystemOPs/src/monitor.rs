use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_usage_percent: f32,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_usage_percent: f32,
    pub network_rx: u64,
    pub network_tx: u64,
    pub processes_count: usize,
}

pub struct ResourceMonitor {
    last_network_stats: Option<(u64, u64)>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            last_network_stats: None,
        }
    }

    pub fn collect_metrics(&mut self) -> SystemMetrics {
        let timestamp = Utc::now();

        let cpu_usage = self.get_cpu_usage();
        let (memory_used, memory_total, memory_usage_percent) = self.get_memory_info();
        let (disk_used, disk_total, disk_usage_percent) = self.get_disk_info();
        let (network_rx, network_tx) = self.get_network_stats();
        let processes_count = self.get_process_count();

        SystemMetrics {
            timestamp,
            cpu_usage,
            memory_used,
            memory_total,
            memory_usage_percent,
            disk_used,
            disk_total,
            disk_usage_percent,
            network_rx,
            network_tx,
            processes_count,
        }
    }

    fn get_cpu_usage(&self) -> f32 {
        let output = Command::new("powershell")
            .args(&[
                "Get-WmiObject Win32_Processor | Measure-Object -Property LoadPercentage -Average | Select-Object -ExpandProperty Average"
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.trim().parse().unwrap_or(0.0)
            }
            _ => {
                eprintln!("Ошибка получения CPU usage");
                0.0
            }
        }
    }

    fn get_memory_info(&self) -> (u64, u64, f32) {
        let output = Command::new("powershell")
            .args(&[
                "$mem = Get-WmiObject Win32_OperatingSystem;",
                "$total = $mem.TotalVisibleMemorySize * 1KB;",
                "$free = $mem.FreePhysicalMemory * 1KB;",
                "$used = $total - $free;",
                "$usage = ($used / $total) * 100;",
                "Write-Output \"$total $used $usage\""
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
                if parts.len() == 3 {
                    let total = parts[0].parse().unwrap_or(0);
                    let used = parts[1].parse().unwrap_or(0);
                    let usage = parts[2].parse().unwrap_or(0.0);
                    return (used, total, usage);
                }
            }
            _ => eprintln!("Ошибка получения memory info"),
        }
        (0, 0, 0.0)
    }

    fn get_disk_info(&self) -> (u64, u64, f32) {
        let output = Command::new("powershell")
            .args(&[
                "$disk = Get-WmiObject Win32_LogicalDisk -Filter \"DeviceID='C:'\";",
                "$total = $disk.Size;",
                "$free = $disk.FreeSpace;",
                "$used = $total - $free;",
                "$usage = ($used / $total) * 100;",
                "Write-Output \"$total $used $usage\""
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
                if parts.len() == 3 {
                    let total = parts[0].parse().unwrap_or(0);
                    let used = parts[1].parse().unwrap_or(0);
                    let usage = parts[2].parse().unwrap_or(0.0);
                    return (used, total, usage);
                }
            }
            _ => eprintln!("Ошибка получения disk info"),
        }
        (0, 0, 0.0)
    }

    fn get_network_stats(&mut self) -> (u64, u64) {
        let output = Command::new("powershell")
            .args(&[
                "$adapters = Get-NetAdapter -Physical | Where-Object {$_.Status -eq 'Up'};",
                "$totalRx = 0; $totalTx = 0;",
                "foreach ($adapter in $adapters) {",
                "    $stats = Get-NetAdapterStatistics -Name $adapter.Name;",
                "    $totalRx += $stats.ReceivedBytes;",
                "    $totalTx += $stats.SentBytes;",
                "}",
                "Write-Output \"$totalRx $totalTx\""
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
                if parts.len() == 2 {
                    let rx: u64 = parts[0].parse().unwrap_or(0);
                    let tx: u64 = parts[1].parse().unwrap_or(0);

                    let result = if let Some((last_rx, last_tx)) = self.last_network_stats {
                        (rx.saturating_sub(last_rx), tx.saturating_sub(last_tx))
                    } else {
                        (0, 0)
                    };

                    self.last_network_stats = Some((rx, tx));
                    return result;
                }
            }
            _ => eprintln!("Ошибка получения network stats"),
        }
        (0, 0)
    }

    fn get_process_count(&self) -> usize {
        let output = Command::new("powershell")
            .args(&["Get-Process | Measure-Object | Select-Object -ExpandProperty Count"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.trim().parse().unwrap_or(0)
            }
            _ => {
                eprintln!("Ошибка получения process count");
                0
            }
        }
    }

    pub fn check_anomalies(&self, metrics: &SystemMetrics) -> Vec<String> {
        let mut anomalies = Vec::new();

        if metrics.cpu_usage > 90.0 {
            anomalies.push(format!("Высокая загрузка CPU: {:.1}%", metrics.cpu_usage));
        }

        if metrics.memory_usage_percent > 90.0 {
            anomalies.push(format!("Высокая загрузка памяти: {:.1}%", metrics.memory_usage_percent));
        }

        if metrics.disk_usage_percent > 90.0 {
            anomalies.push(format!("Высокая загрузка диска: {:.1}%", metrics.disk_usage_percent));
        }

        anomalies
    }
}