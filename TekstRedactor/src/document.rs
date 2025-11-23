use std::path::{Path, PathBuf};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub content: String,
    path: Option<PathBuf>,
    modified: bool,

    // Undo/Redo
    undo_stack: VecDeque<String>,
    redo_stack: VecDeque<String>,
    max_undo_steps: usize,

    // Find/Replace
    pub find_text: String,
    pub replace_text: String,
    pub match_case: bool,
    pub whole_word: bool,
    pub current_find_pos: usize,

    // Cursor position
    cursor_position: usize,
    selection: Option<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub pages: usize,
    pub words: usize,
    pub characters: usize,
    pub characters_no_spaces: usize,
    pub lines: usize,
    pub paragraphs: usize,
}

impl Document {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: String::new(),
            path: None,
            modified: false,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_steps: 100,
            find_text: String::new(),
            replace_text: String::new(),
            match_case: false,
            whole_word: false,
            current_find_pos: 0,
            cursor_position: 0,
            selection: None,
        }
    }

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let title = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Ok(Self {
            title,
            content,
            path: Some(path.to_path_buf()),
            modified: false,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_steps: 100,
            find_text: String::new(),
            replace_text: String::new(),
            match_case: false,
            whole_word: false,
            current_find_pos: 0,
            cursor_position: 0,
            selection: None,
        })
    }

    pub fn save(&mut self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::write(path, &self.content)?;
        self.path = Some(path.to_path_buf());
        self.modified = false;
        self.title = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();
        Ok(())
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.save(path)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    // Undo/Redo functionality
    pub fn push_undo_state(&mut self) {
        if self.undo_stack.len() >= self.max_undo_steps {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(self.content.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(previous_state) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(self.content.clone());
            self.content = previous_state;
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(self.content.clone());
            self.content = next_state;
            self.modified = true;
        }
    }

    // Find/Replace functionality
    pub fn find_next(&mut self) -> bool {
        if self.find_text.is_empty() {
            return false;
        }

        let content = if self.match_case {
            self.content.clone()
        } else {
            self.content.to_lowercase()
        };

        let find_text = if self.match_case {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };

        if let Some(pos) = content[self.current_find_pos..].find(&find_text) {
            self.current_find_pos += pos;
            self.selection = Some((self.current_find_pos, self.current_find_pos + find_text.len()));
            self.current_find_pos += find_text.len();
            true
        } else {
            self.current_find_pos = 0;
            false
        }
    }

    pub fn replace_next(&mut self) -> bool {
        if let Some((start, end)) = self.selection {
            if self.content[start..end] == self.find_text ||
                (!self.match_case && self.content[start..end].eq_ignore_ascii_case(&self.find_text)) {

                self.push_undo_state();
                self.content.replace_range(start..end, &self.replace_text);
                self.modified = true;

                // Adjust selection to replaced text
                self.selection = Some((start, start + self.replace_text.len()));
                self.current_find_pos = start + self.replace_text.len();

                return true;
            }
        }

        self.find_next()
    }

    pub fn replace_all(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        self.push_undo_state();
        let count = if self.match_case {
            let matches = self.content.matches(&self.find_text).count();
            self.content = self.content.replace(&self.find_text, &self.replace_text);
            matches
        } else {
            // Case-insensitive replacement
            let mut result = String::new();
            let mut last_end = 0;
            let find_lower = self.find_text.to_lowercase();
            let content_lower = self.content.to_lowercase();
            let mut count = 0;

            while let Some(start) = content_lower[last_end..].find(&find_lower) {
                let start = last_end + start;
                let end = start + self.find_text.len();

                result.push_str(&self.content[last_end..start]);
                result.push_str(&self.replace_text);
                last_end = end;
                count += 1;
            }

            result.push_str(&self.content[last_end..]);
            self.content = result;
            count
        };

        if count > 0 {
            self.modified = true;
        }
    }

    // Copy/Cut/Paste functionality
    pub fn copy(&self) {
        if let Some((start, end)) = self.selection {
            let selected_text = &self.content[start..end];
            // For now, we'll just print since clipboard might have issues
            println!("Copied: {}", selected_text);
        }
    }

    pub fn cut(&mut self) {
        if let Some((start, end)) = self.selection {
            let selected_text = &self.content[start..end].to_string();
            println!("Cut: {}", selected_text); // Simulate clipboard

            self.push_undo_state();
            self.content.replace_range(start..end, "");
            self.modified = true;
            self.selection = None;
        }
    }

    pub fn paste(&mut self) {
        // Simulate paste - in real implementation, use arboard
        let text = "pasted_text"; // This would come from clipboard
        if let Some((start, end)) = self.selection {
            self.push_undo_state();
            self.content.replace_range(start..end, text);
        } else {
            self.push_undo_state();
            self.content.insert_str(self.cursor_position, text);
        }
        self.modified = true;
    }

    pub fn select_all(&mut self) {
        self.selection = Some((0, self.content.len()));
    }

    // Cursor and selection management
    pub fn cursor_line(&self) -> usize {
        self.content[..self.cursor_position].matches('\n').count() + 1
    }

    pub fn cursor_column(&self) -> usize {
        if let Some(last_newline) = self.content[..self.cursor_position].rfind('\n') {
            self.cursor_position - last_newline - 1
        } else {
            self.cursor_position
        }
    }

    // Statistics
    pub fn calculate_stats(&self) -> DocumentStats {
        let characters = self.content.chars().count();
        let characters_no_spaces = self.content.chars().filter(|c| !c.is_whitespace()).count();
        let words = self.content.split_whitespace().count();
        let lines = self.content.lines().count();
        let paragraphs = self.content.split("\n\n").count();

        // Estimate pages (assuming ~500 words per page)
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
}