use eframe::egui::{
    self, menu, Color32, Context, FontId,
    Key, Modifiers, RichText, ViewportCommand
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Document {
    title: String,
    content: String,
    path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    last_content: String, // Перемещаем last_content в Document
}

impl Document {
    fn new(title: &str) -> Self {
        let content = String::new();
        Self {
            title: title.to_string(),
            content: content.clone(),
            path: None,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_content: content,
        }
    }

    fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Безымянный")
            .to_string();

        Ok(Self {
            title,
            content: content.clone(),
            path: Some(path.to_path_buf()),
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_content: content,
        })
    }

    fn save(&mut self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::write(path, &self.content)?;
        self.path = Some(path.to_path_buf());
        self.modified = false;
        self.title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Безымянный")
            .to_string();
        Ok(())
    }

    fn save_as(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.save(path)
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    fn save_state_before_change(&mut self) {
        self.undo_stack.push(self.content.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.redo_stack.push(self.content.clone());
            self.content = previous_state;
            self.modified = true;
            self.last_content = self.content.clone();
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.content.clone());
            self.content = next_state;
            self.modified = true;
            self.last_content = self.content.clone();
            true
        } else {
            false
        }
    }

    fn calculate_stats(&self) -> DocumentStats {
        let characters = self.content.chars().count();
        let characters_no_spaces = self.content.chars().filter(|c| !c.is_whitespace()).count();
        let words = self.content.split_whitespace().count();
        let lines = self.content.lines().count();
        let paragraphs = self.content.split("\n\n").count();

        let pages = (words as f32 / 500.0).ceil() as usize;

        DocumentStats {
            pages,
            words,
            characters,
            characters_no_spaces,
            lines,
            paragraphs,
        }
    }

    fn cursor_line(&self) -> usize {
        self.content[..].matches('\n').count() + 1
    }

    fn cursor_column(&self) -> usize {
        self.content.len().saturating_sub(
            self.content.rfind('\n').map(|pos| pos + 1).unwrap_or(0)
        )
    }

    fn update_last_content(&mut self) {
        // Сохраняем предыдущее состояние в стек отмены, если содержимое изменилось
        if self.content != self.last_content {
            if !self.undo_stack.last().map_or(false, |last| last == &self.last_content) {
                self.undo_stack.push(self.last_content.clone());
                if self.undo_stack.len() > 50 {
                    self.undo_stack.remove(0);
                }
                self.redo_stack.clear();
            }
            self.last_content = self.content.clone();
            self.modified = true;
        }
    }
}

struct DocumentStats {
    pages: usize,
    words: usize,
    characters: usize,
    characters_no_spaces: usize,
    lines: usize,
    paragraphs: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn all() -> [Theme; 2] {
        [Theme::Light, Theme::Dark]
    }

    fn egui_visuals(&self) -> egui::Visuals {
        match self {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        }
    }
}

#[derive(Clone)]
struct AppSettings {
    theme: Theme,
    font_size: f32,
    auto_save_enabled: bool,
    auto_save_interval: Duration,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            font_size: 16.0,
            auto_save_enabled: true,
            auto_save_interval: Duration::from_secs(30),
        }
    }
}

impl AppSettings {
    fn load() -> Self {
        Self::default()
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

pub struct TextEditorApp {
    documents: Vec<Document>,
    active_document: usize,
    settings: AppSettings,

    show_settings: bool,
    show_stats: bool,
    show_find_replace: bool,
    error_message: Option<String>,
    last_save_time: Instant,

    find_text: String,
    replace_text: String,
    match_case: bool,
    whole_word: bool,
}

impl Default for TextEditorApp {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            active_document: 0,
            settings: AppSettings::default(),
            show_settings: false,
            show_stats: false,
            show_find_replace: false,
            error_message: None,
            last_save_time: Instant::now(),
            find_text: String::new(),
            replace_text: String::new(),
            match_case: false,
            whole_word: false,
        }
    }
}

impl TextEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.settings = AppSettings::load();
        app.apply_settings(&cc.egui_ctx);

        app.documents.push(Document::new("Безымянный 1"));

        app
    }

    fn apply_settings(&self, ctx: &Context) {
        ctx.set_visuals(self.settings.theme.egui_visuals());
    }

    fn ensure_active_document(&mut self) {
        if self.documents.is_empty() {
            self.documents.push(Document::new("Безымянный 1"));
        }
        if self.active_document >= self.documents.len() {
            self.active_document = self.documents.len().saturating_sub(1);
        }
    }

    fn current_document_mut(&mut self) -> &mut Document {
        &mut self.documents[self.active_document]
    }

    fn current_document(&self) -> &Document {
        &self.documents[self.active_document]
    }

    fn new_document(&mut self) {
        let count = self.documents.len() + 1;
        self.documents.push(Document::new(&format!("Безымянный {}", count)));
        self.active_document = self.documents.len() - 1;
    }

    fn open_document(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Текстовые файлы", &["txt", "md", "rs", "json", "xml", "html", "css"])
            .add_filter("Все файлы", &["*"])
            .pick_file()
        {
            match Document::load(&path) {
                Ok(doc) => {
                    self.documents.push(doc);
                    self.active_document = self.documents.len() - 1;
                }
                Err(e) => {
                    self.error_message = Some(format!("Не удалось открыть файл: {}", e));
                }
            }
        }
    }

    fn save_document(&mut self) {
        let path = {
            let doc = self.current_document();
            doc.path().map(|p| p.to_path_buf())
        };

        if let Some(path) = path {
            let doc = self.current_document_mut();
            if let Err(e) = doc.save(&path) {
                self.error_message = Some(format!("Не удалось сохранить файл: {}", e));
            } else {
                self.last_save_time = Instant::now();
                println!("Файл сохранен: {:?}", path);
            }
        } else {
            self.save_document_as();
        }
    }

    fn save_document_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Текстовые файлы", &["txt"])
            .add_filter("Все файлы", &["*"])
            .save_file()
        {
            let path = if path.extension().is_none() {
                path.with_extension("txt")
            } else {
                path
            };

            let doc = self.current_document_mut();
            if let Err(e) = doc.save_as(&path) {
                self.error_message = Some(format!("Не удалось сохранить файл: {}", e));
            } else {
                self.last_save_time = Instant::now();
                println!("Файл сохранен как: {:?}", path);
            }
        }
    }

    fn close_current_document(&mut self) {
        if self.documents.len() > 1 {
            self.documents.remove(self.active_document);
            self.active_document = self.active_document.saturating_sub(1);
        }
    }

    fn auto_save(&mut self) {
        if self.settings.auto_save_enabled && self.last_save_time.elapsed() > self.settings.auto_save_interval {
            let paths_to_save: Vec<PathBuf> = self.documents
                .iter()
                .filter(|doc| doc.is_modified())
                .filter_map(|doc| doc.path().map(|p| p.to_path_buf()))
                .collect();

            for path in paths_to_save {
                for doc in &mut self.documents {
                    if let Some(doc_path) = doc.path() {
                        if doc_path == path.as_path() && doc.is_modified() {
                            let _ = doc.save(&path);
                            break;
                        }
                    }
                }
            }
            self.last_save_time = Instant::now();
        }
    }

    fn copy_text(&self) {
        let doc = self.current_document();
        println!("Текст скопирован: {}", doc.content);
    }

    fn cut_text(&mut self) {
        let doc = self.current_document_mut();
        if !doc.content.is_empty() {
            doc.save_state_before_change();
            let old_content = std::mem::take(&mut doc.content);
            println!("Текст вырезан: {}", old_content);
            doc.set_modified(true);
        }
    }

    fn paste_text(&mut self) {
        let doc = self.current_document_mut();
        doc.save_state_before_change();
        doc.content.push_str("[ВСТАВЛЕННЫЙ ТЕКСТ]");
        doc.set_modified(true);
    }

    fn select_all(&mut self) {
        println!("Выделить всё");
    }

    fn show_menu_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("Файл", |ui| {
                    if ui.button("Создать").clicked() {
                        self.new_document();
                        ui.close_menu();
                    }
                    if ui.button("Открыть...").clicked() {
                        self.open_document();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Сохранить").clicked() {
                        self.save_document();
                        ui.close_menu();
                    }
                    if ui.button("Сохранить как...").clicked() {
                        self.save_document_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Закрыть").clicked() {
                        self.close_current_document();
                        ui.close_menu();
                    }
                    if ui.button("Выход").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Правка", |ui| {
                    let can_undo = !self.current_document().undo_stack.is_empty();
                    let can_redo = !self.current_document().redo_stack.is_empty();

                    if ui.add_enabled(can_undo, egui::Button::new("Отменить")).clicked() {
                        self.current_document_mut().undo();
                        ui.close_menu();
                    }
                    if ui.add_enabled(can_redo, egui::Button::new("Повторить")).clicked() {
                        self.current_document_mut().redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Вырезать").clicked() {
                        self.cut_text();
                        ui.close_menu();
                    }
                    if ui.button("Копировать").clicked() {
                        self.copy_text();
                        ui.close_menu();
                    }
                    if ui.button("Вставить").clicked() {
                        self.paste_text();
                        ui.close_menu();
                    }
                    if ui.button("Выделить всё").clicked() {
                        self.select_all();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Найти/Заменить").clicked() {
                        self.show_find_replace = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Вид", |ui| {
                    if ui.button("Статистика документа").clicked() {
                        self.show_stats = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Увеличить").clicked() {
                        self.settings.font_size = (self.settings.font_size + 1.0).min(72.0);
                        ui.close_menu();
                    }
                    if ui.button("Уменьшить").clicked() {
                        self.settings.font_size = (self.settings.font_size - 1.0).max(8.0);
                        ui.close_menu();
                    }
                    if ui.button("Сбросить масштаб").clicked() {
                        self.settings.font_size = 16.0;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Настройки", |ui| {
                    if ui.button("Параметры...").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📄 Создать").clicked() {
                self.new_document();
            }
            if ui.button("📂 Открыть").clicked() {
                self.open_document();
            }
            if ui.button("💾 Сохранить").clicked() {
                self.save_document();
            }
            if ui.button("💾 Сохранить как").clicked() {
                self.save_document_as();
            }
            ui.separator();

            let can_undo = !self.current_document().undo_stack.is_empty();
            let can_redo = !self.current_document().redo_stack.is_empty();

            if ui.add_enabled(can_undo, egui::Button::new("↶ Отменить")).clicked() {
                self.current_document_mut().undo();
            }
            if ui.add_enabled(can_redo, egui::Button::new("↷ Повторить")).clicked() {
                self.current_document_mut().redo();
            }
            ui.separator();
            if ui.button("🔍 Найти").clicked() {
                self.show_find_replace = true;
            }
        });
    }

    fn show_document_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (i, doc) in self.documents.iter().enumerate() {
                let is_active = i == self.active_document;
                let label = if doc.is_modified() {
                    format!("{} ●", doc.title())
                } else {
                    doc.title().to_string()
                };

                let response = ui.selectable_label(is_active, label);

                if response.clicked() && !is_active {
                    self.active_document = i;
                }

                if self.documents.len() > 1 {
                    let close_response = ui.small_button("✕");
                    if close_response.clicked() {
                        self.documents.remove(i);
                        self.active_document = self.active_document.saturating_sub(1);
                        break;
                    }
                }
            }

            if ui.button("+").clicked() {
                self.new_document();
            }
        });
    }

    fn show_find_replace_dialog(&mut self, ctx: &Context) {
        if !self.show_find_replace {
            return;
        }

        let mut find_text = self.find_text.clone();
        let mut replace_text = self.replace_text.clone();
        let mut match_case = self.match_case;
        let mut whole_word = self.whole_word;

        let mut find_next_clicked = false;
        let mut replace_clicked = false;
        let mut replace_all_clicked = false;

        egui::Window::new("Найти и заменить")
            .open(&mut self.show_find_replace)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Найти:");
                    ui.text_edit_singleline(&mut find_text);
                });

                ui.horizontal(|ui| {
                    ui.label("Заменить:");
                    ui.text_edit_singleline(&mut replace_text);
                });

                ui.horizontal(|ui| {
                    if ui.button("Найти далее").clicked() {
                        find_next_clicked = true;
                    }
                    if ui.button("Заменить").clicked() {
                        replace_clicked = true;
                    }
                    if ui.button("Заменить все").clicked() {
                        replace_all_clicked = true;
                    }
                });

                ui.checkbox(&mut match_case, "С учетом регистра");
                ui.checkbox(&mut whole_word, "Целое слово");
            });

        self.find_text = find_text;
        self.replace_text = replace_text;
        self.match_case = match_case;
        self.whole_word = whole_word;

        if find_next_clicked {
            let doc = self.current_document();
            if !self.find_text.is_empty() {
                if let Some(pos) = doc.content.find(&self.find_text) {
                    println!("Найдено в позиции: {}", pos);
                }
            }
        }

        if replace_clicked {
            let find_text_clone = self.find_text.clone();
            let replace_text_clone = self.replace_text.clone();
            let doc = self.current_document_mut();
            if !find_text_clone.is_empty() && doc.content.contains(&find_text_clone) {
                doc.save_state_before_change();
                doc.content = doc.content.replacen(&find_text_clone, &replace_text_clone, 1);
                doc.set_modified(true);
            }
        }

        if replace_all_clicked {
            let find_text_clone = self.find_text.clone();
            let replace_text_clone = self.replace_text.clone();
            let doc = self.current_document_mut();
            if !find_text_clone.is_empty() && doc.content.contains(&find_text_clone) {
                doc.save_state_before_change();
                doc.content = doc.content.replace(&find_text_clone, &replace_text_clone);
                doc.set_modified(true);
            }
        }
    }

    fn show_settings_dialog(&mut self, ctx: &Context) {
        if !self.show_settings {
            return;
        }

        let mut font_size = self.settings.font_size;
        let mut theme = self.settings.theme;
        let mut auto_save_enabled = self.settings.auto_save_enabled;
        let mut show_settings = self.show_settings;

        let mut apply_clicked = false;
        let mut cancel_clicked = false;

        egui::Window::new("Настройки")
            .open(&mut show_settings)
            .show(ctx, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Размер шрифта:");
                        ui.add(egui::Slider::new(&mut font_size, 8.0..=72.0));
                        ui.end_row();

                        ui.label("Тема:");
                        egui::ComboBox::from_id_source("theme_combo")
                            .selected_text(format!("{:?}", theme))
                            .show_ui(ui, |ui| {
                                for t in Theme::all() {
                                    ui.selectable_value(&mut theme, t, format!("{:?}", t));
                                }
                            });
                        ui.end_row();

                        ui.label("Автосохранение:");
                        ui.checkbox(&mut auto_save_enabled, "Включено");
                        ui.end_row();
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Применить").clicked() {
                        apply_clicked = true;
                    }
                    if ui.button("Отмена").clicked() {
                        cancel_clicked = true;
                    }
                });
            });

        if cancel_clicked {
            show_settings = false;
        }

        if apply_clicked {
            self.settings.font_size = font_size;
            self.settings.theme = theme;
            self.settings.auto_save_enabled = auto_save_enabled;
            self.apply_settings(ctx);
            let _ = self.settings.save();
            show_settings = false;
        }

        self.show_settings = show_settings;
    }

    fn show_stats_dialog(&mut self, ctx: &Context) {
        if !self.show_stats {
            return;
        }

        let stats = self.current_document().calculate_stats();
        let mut show_stats = self.show_stats;

        egui::Window::new("Статистика документа")
            .open(&mut show_stats)
            .show(ctx, |ui| {
                egui::Grid::new("stats_grid")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Страницы:"); ui.label(format!("{}", stats.pages));
                        ui.end_row();
                        ui.label("Слова:"); ui.label(format!("{}", stats.words));
                        ui.end_row();
                        ui.label("Символы:"); ui.label(format!("{}", stats.characters));
                        ui.end_row();
                        ui.label("Строки:"); ui.label(format!("{}", stats.lines));
                        ui.end_row();
                    });
            });

        self.show_stats = show_stats;
    }

    fn show_error_dialog(&mut self, ctx: &Context) {
        if let Some(error) = &self.error_message {
            let error_clone = error.clone();
            let mut error_message = self.error_message.clone();

            egui::Window::new("Ошибка")
                .open(&mut error_message.is_some())
                .show(ctx, |ui| {
                    ui.label(RichText::new(error_clone).color(Color32::RED));
                    ui.separator();
                    if ui.button("OK").clicked() {
                        error_message = None;
                    }
                });

            self.error_message = error_message;
        }
    }

    fn show_status_bar(&self, ui: &mut egui::Ui) {
        let doc = self.current_document();
        let stats = doc.calculate_stats();

        ui.horizontal(|ui| {
            ui.label(format!(
                "Строка {}, Колонка {} | Слова: {} | Символы: {}",
                doc.cursor_line(), doc.cursor_column(), stats.words, stats.characters
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if doc.is_modified() {
                    ui.label(RichText::new("Изменен").color(Color32::YELLOW));
                }
                ui.label("UTF-8");
            });
        });
    }
}

impl eframe::App for TextEditorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.ensure_active_document();
        self.auto_save();

        // Обработка горячих клавиш
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::CTRL, Key::N) {
                self.new_document();
            }
            if i.consume_key(Modifiers::CTRL, Key::O) {
                self.open_document();
            }
            if i.consume_key(Modifiers::CTRL, Key::S) {
                self.save_document();
            }
            if i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::S) {
                self.save_document_as();
            }
            if i.consume_key(Modifiers::CTRL, Key::F) {
                self.show_find_replace = true;
            }
            if i.consume_key(Modifiers::CTRL, Key::A) {
                self.select_all();
            }
            if i.consume_key(Modifiers::CTRL, Key::C) {
                self.copy_text();
            }
            if i.consume_key(Modifiers::CTRL, Key::X) {
                self.cut_text();
            }
            if i.consume_key(Modifiers::CTRL, Key::V) {
                self.paste_text();
            }
            if i.consume_key(Modifiers::CTRL, Key::Z) {
                self.current_document_mut().undo();
            }
            if i.consume_key(Modifiers::CTRL, Key::Y) {
                self.current_document_mut().redo();
            }
        });

        self.show_menu_bar(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.show_toolbar(ui);
        });

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            self.show_document_tabs(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let font_size = self.settings.font_size;
            let doc = self.current_document_mut();

            let response = egui::ScrollArea::vertical()
                .id_source("text_editor")
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut doc.content)
                        .font(FontId::monospace(font_size))
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .lock_focus(true);

                    ui.add(text_edit)
                });

            // Обновляем состояние undo/redo после изменений
            if response.inner.changed() {
                doc.update_last_content();
            }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        self.show_find_replace_dialog(ctx);
        self.show_settings_dialog(ctx);
        self.show_stats_dialog(ctx);
        self.show_error_dialog(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.settings.save();
    }
}