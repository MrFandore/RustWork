use serde::{Deserialize, Serialize};
use std::time::Duration;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn all() -> [Theme; 2] {
        [Theme::Light, Theme::Dark]
    }

    pub fn egui_visuals(&self) -> egui::Visuals {
        match self {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub font_size: f32,
    pub auto_save_enabled: bool,
    pub auto_save_interval_secs: u64,
    #[serde(skip)]
    pub auto_save_interval: Duration,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            font_size: 16.0,
            auto_save_enabled: true,
            auto_save_interval_secs: 30,
            auto_save_interval: Duration::from_secs(30),
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // For now, just return default settings
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just succeed
        Ok(())
    }
}