mod config;
mod monitor;
mod storage;
mod service_manager;
mod notification;
mod security;

use std::sync::Arc;
use tokio::sync::RwLock;
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::monitor::ResourceMonitor;
use crate::storage::Storage;
use crate::service_manager::WindowsServiceManager;
use crate::notification::NotificationSystem;
use crate::security::SecurityManager;

#[derive(Parser)]
#[command(name = "MonitorSystemOPs")]
#[command(about = "System Operations Monitoring Service", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Uninstall,
    Start,
    Stop,
    Restart,
    Status,
    Run,
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => {
            WindowsServiceManager::install()?;
        }
        Some(Commands::Uninstall) => {
            WindowsServiceManager::uninstall()?;
        }
        Some(Commands::Start) => {
            WindowsServiceManager::start()?;
        }
        Some(Commands::Stop) => {
            WindowsServiceManager::stop()?;
        }
        Some(Commands::Restart) => {
            WindowsServiceManager::restart()?;
        }
        Some(Commands::Status) => {
            WindowsServiceManager::status()?;
        }
        Some(Commands::Config) => {
            Config::generate_default()?;
        }
        Some(Commands::Run) | None => {
            run_service().await?;
        }
    }

    Ok(())
}

async fn run_service() -> anyhow::Result<()> {
    println!("🚀 Запуск MonitorSystemOPs...");

    let config = Config::load().unwrap_or_else(|_| {
        println!("Используется конфигурация по умолчанию");
        Config::generate_default().unwrap();
        Config::load().unwrap()
    });

    let storage = Arc::new(Storage::new());
    let current_metrics = Arc::new(RwLock::new(None));

    {
        let storage = storage.clone();
        let current_metrics = current_metrics.clone();
        let host = config.web.host.clone();
        let port = config.web.port;

        tokio::spawn(async move {
            if let Err(e) = start_simple_web_server(storage, current_metrics, host, port).await {
                eprintln!("Ошибка веб-сервера: {}", e);
            }
        });
    }

    let mut monitor = ResourceMonitor::new();
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(config.monitoring.interval_seconds)
    );

    println!("📊 Мониторинг запущен. Интервал: {} сек.", config.monitoring.interval_seconds);
    println!("🌐 Веб-интерфейс: http://{}:{}", config.web.host, config.web.port);

    loop {
        interval.tick().await;

        let metrics = monitor.collect_metrics();
        let metrics_log = metrics.clone();

        let anomalies = monitor.check_anomalies(&metrics);
        if !anomalies.is_empty() {
            println!("⚠️  Предупреждение: {}", anomalies.join(", "));
        }

        if let Err(e) = storage.save_metrics(&metrics) {
            eprintln!("❌ Ошибка сохранения: {}", e);
        }

        {
            let mut current = current_metrics.write().await;
            *current = Some(metrics);
        }

        if let Err(e) = storage.cleanup_old_records(config.storage.max_records) {
            eprintln!("❌ Ошибка очистки: {}", e);
        }

        println!("📈 CPU: {:.1}%, Memory: {:.1}%, Disk: {:.1}%",
                 metrics_log.cpu_usage,
                 metrics_log.memory_usage_percent,
                 metrics_log.disk_usage_percent);
    }
}

async fn start_simple_web_server(
    storage: Arc<Storage>,
    current_metrics: Arc<RwLock<Option<crate::monitor::SystemMetrics>>>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    use warp::Filter;
    use std::net::SocketAddr;

    let storage_filter = warp::any().map(move || storage.clone());
    let metrics_filter = warp::any().map(move || current_metrics.clone());

    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(metrics_filter)
        .and_then(|metrics: Arc<RwLock<Option<crate::monitor::SystemMetrics>>>| async move {
            let metrics_guard = metrics.read().await;
            match &*metrics_guard {
                Some(m) => Ok(warp::reply::json(m)),
                None => Err(warp::reject::not_found()),
            }
        });

    let history_route = warp::path("history")
        .and(warp::get())
        .and(storage_filter)
        .and_then(|storage: Arc<Storage>| async move {
            match storage.load_metrics() {
                Ok(metrics) => Ok(warp::reply::json(&metrics)),
                Err(_) => Err(warp::reject::not_found()),
            }
        });

    let index_route = warp::path::end()
        .and(warp::get())
        .map(|| {
            warp::reply::html(include_str!("../static/simple_index.html"))
        });

    let routes = index_route
        .or(metrics_route)
        .or(history_route)
        .with(warp::cors().allow_any_origin());

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("🌐 Веб-сервер запущен на http://{}", addr);
    warp::serve(routes).run(addr).await;

    Ok(())
}