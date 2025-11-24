use windows_service::{
    service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager as WinServiceManager, ServiceManagerAccess},
};
use std::ffi::OsString;
use anyhow::Result;

const SERVICE_NAME: &str = "MonitorSystemOPs";
const SERVICE_DISPLAY_NAME: &str = "System Operations Monitor";
const SERVICE_DESCRIPTION: &str = "Monitors system resources and provides operational insights";

pub struct WindowsServiceManager;

impl WindowsServiceManager {
    pub fn install() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = WinServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_binary_path = std::env::current_exe()?;

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: service_binary_path,
            launch_arguments: vec![OsString::from("--service")],
            dependencies: vec![],
            account_name: Some(OsString::from("NT AUTHORITY\\LocalService")),
            account_password: None,
        };

        let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;

        // Устанавливаем описание службы
        service.set_description(SERVICE_DESCRIPTION)?;

        println!("Служба '{}' успешно установлена", SERVICE_NAME);
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = WinServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
        let service = service_manager.open_service(SERVICE_NAME, service_access)?;

        // Останавливаем службу если запущена
        if let Ok(status) = service.query_status() {
            if status.current_state != ServiceState::Stopped {
                service.stop()?;
                println!("Остановка службы...");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }

        service.delete()?;
        println!("Служба '{}' успешно удалена", SERVICE_NAME);
        Ok(())
    }

    pub fn start() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = WinServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access = ServiceAccess::START;
        let service = service_manager.open_service(SERVICE_NAME, service_access)?;

        service.start(&[] as &[OsString])?;
        println!("Служба '{}' запущена", SERVICE_NAME);
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = WinServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access = ServiceAccess::STOP;
        let service = service_manager.open_service(SERVICE_NAME, service_access)?;

        service.stop()?;
        println!("Служба '{}' остановлена", SERVICE_NAME);
        Ok(())
    }

    pub fn status() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = WinServiceManager::local_computer(None::<&str>, manager_access)?;

        let service_access = ServiceAccess::QUERY_STATUS;
        match service_manager.open_service(SERVICE_NAME, service_access) {
            Ok(service) => {
                let status = service.query_status()?;
                println!("Служба: {}", SERVICE_NAME);
                println!("Отображаемое имя: {}", SERVICE_DISPLAY_NAME);
                println!("Статус: {:?}", status.current_state);
                // Исправляем вывод PID - используем форматирование для Option
                if let Some(pid) = status.process_id {
                    println!("PID: {}", pid);
                } else {
                    println!("PID: не доступен");
                }
                println!("Тип: {:?}", status.service_type);
            }
            Err(_) => {
                println!("Служба '{}' не установлена", SERVICE_NAME);
            }
        }

        Ok(())
    }

    pub fn restart() -> Result<()> {
        Self::stop()?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        Self::start()?;
        println!("Служба '{}' перезапущена", SERVICE_NAME);
        Ok(())
    }
}