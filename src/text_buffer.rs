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
}

/// Simple implementation that wraps a Vec<String>
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
}
