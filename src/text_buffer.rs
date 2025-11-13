use std::ops::Range;

/// A minimal text buffer trait that supports the features we have so far
pub trait TextBuffer {
    /// Get the total number of lines in the buffer
    fn line_count(&self) -> usize;

    /// Get a specific line by index (0-based)
    fn get_line(&self, line_idx: usize) -> Option<&str>;

    /// Get the length of a specific line in characters
    fn line_len(&self, line_idx: usize) -> usize {
        self.get_line(line_idx).map(|s| s.len()).unwrap_or(0)
    }

    /// Get all lines (for now, while we're simple)
    fn all_lines(&self) -> Vec<String>;

    /// Insert text at a specific position (row, col)
    fn insert_at(&mut self, row: usize, col: usize, text: &str);

    /// Delete a character at a specific position (row, col)
    fn delete_at(&mut self, row: usize, col: usize);

    /// Delete backwards from a specific position (row, col)
    fn backspace_at(&mut self, row: usize, col: usize);
}

/// Simple implementation that wraps a Vec<String>
#[derive(Clone)]
pub struct SimpleBuffer {
    lines: Vec<String>,
}

impl SimpleBuffer {
    pub fn new(lines: Vec<String>) -> Self {
        // Ensure we always have at least one line
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self { lines }
    }

    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        Self::new(lines)
    }
}

impl TextBuffer for SimpleBuffer {
    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn get_line(&self, line_idx: usize) -> Option<&str> {
        self.lines.get(line_idx).map(|s| s.as_str())
    }

    fn all_lines(&self) -> Vec<String> {
        self.lines.clone()
    }

    fn insert_at(&mut self, row: usize, col: usize, text: &str) {
        if row >= self.lines.len() {
            return;
        }

        let col = col.min(self.lines[row].len());

        if text.contains('\n') {
            // Handle multi-line insert
            let new_lines: Vec<&str> = text.split('\n').collect();
            let current_line = &self.lines[row];
            let first_part = current_line[..col].to_string();
            let last_part = current_line[col..].to_string();

            // Update current line
            self.lines[row] = format!("{}{}", first_part, new_lines[0]);

            // Insert middle lines if any
            for i in 1..new_lines.len() - 1 {
                self.lines.insert(row + i, new_lines[i].to_string());
            }

            // Insert last line if there was a split
            if new_lines.len() > 1 {
                let last_idx = new_lines.len() - 1;
                self.lines.insert(
                    row + last_idx,
                    format!("{}{}", new_lines[last_idx], last_part),
                );
            }
        } else {
            // Simple single-line insert
            self.lines[row].insert_str(col, text);
        }
    }

    fn delete_at(&mut self, row: usize, col: usize) {
        if row >= self.lines.len() {
            return;
        }

        let line = &self.lines[row];
        if col < line.len() {
            self.lines[row].remove(col);
        } else if row < self.lines.len() - 1 {
            // At end of line, merge with next line
            let next_line = self.lines.remove(row + 1);
            self.lines[row].push_str(&next_line);
        }
    }

    fn backspace_at(&mut self, row: usize, col: usize) {
        if row >= self.lines.len() {
            return;
        }

        if col > 0 {
            self.lines[row].remove(col - 1);
        } else if row > 0 {
            // At beginning of line, merge with previous line
            let current_line = self.lines.remove(row);
            self.lines[row - 1].push_str(&current_line);
        }
    }
}
