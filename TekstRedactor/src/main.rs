mod app;

use eframe::NativeOptions;
use app::TextEditorApp;
use anyhow::Result;

fn main() -> Result<(), eframe::Error> {
    let native_options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Редактор им. Жмыха Ящерицы")
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon().unwrap()), // Добавляем иконку
        ..Default::default()
    };

    eframe::run_native(
        "Редактор им. Жмыха Ящерицы",
        native_options,
        Box::new(|cc| Box::new(TextEditorApp::new(cc))),
    )
}

// Функция для загрузки иконки
fn load_icon() -> anyhow::Result<eframe::egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)?;
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_vec();
    let width = image.width();
    let height = image.height();

    Ok(eframe::egui::IconData {
        rgba: pixels,
        width,
        height,
    })
}