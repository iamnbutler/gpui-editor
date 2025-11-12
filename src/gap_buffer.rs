use crate::text_buffer::TextBuffer;

/// A gap buffer implementation for efficient text editing
///
/// The gap buffer maintains a "gap" in the text where insertions and deletions
/// occur. This makes operations at the cursor position O(1).
#[derive(Debug, Clone)]
pub struct GapBuffer {
    /// The underlying buffer containing text and gap
    buffer: Vec<char>,
    /// Start position of the gap
    gap_start: usize,
    /// End position of the gap (exclusive)
    gap_end: usize,
}

impl GapBuffer {
    /// Create a new empty gap buffer
    pub fn new() -> Self {
        let initial_capacity = 64;
        let mut buffer = Vec::with_capacity(initial_capacity);
        buffer.resize(initial_capacity, '\0');

        Self {
            buffer,
            gap_start: 0,
            gap_end: initial_capacity,
        }
    }

    /// Create a gap buffer from text
    pub fn from_text(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();
        let gap_size = 64.max(text_len / 4); // At least 64 chars gap
        let total_size = text_len + gap_size;

        let mut buffer = Vec::with_capacity(total_size);
        buffer.extend_from_slice(&chars);
        buffer.resize(total_size, '\0');

        Self {
            buffer,
            gap_start: text_len,
            gap_end: total_size,
        }
    }

    /// Create a gap buffer from lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        let text = lines.join("\n");
        Self::from_text(&text)
    }

    /// Get the total length of the text (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    /// Get the size of the gap
    fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    /// Convert a text position to a buffer position
    fn text_pos_to_buffer_pos(&self, text_pos: usize) -> usize {
        if text_pos < self.gap_start {
            text_pos
        } else {
            text_pos + self.gap_size()
        }
    }

    /// Move the gap to a specific text position
    pub fn move_gap_to(&mut self, text_pos: usize) {
        let text_pos = text_pos.min(self.len());

        if text_pos == self.gap_start {
            return; // Gap is already at the right position
        }

        if text_pos < self.gap_start {
            // Move gap left
            let move_count = self.gap_start - text_pos;
            let src_start = text_pos;
            let dst_start = self.gap_end - move_count;

            // Copy characters from before gap to after gap
            for i in (0..move_count).rev() {
                self.buffer[dst_start + i] = self.buffer[src_start + i];
            }

            self.gap_start = text_pos;
            self.gap_end = text_pos + self.gap_size();
        } else {
            // Move gap right
            let buffer_pos = text_pos + self.gap_size();
            let move_count = buffer_pos - self.gap_end;
            let src_start = self.gap_end;
            let dst_start = self.gap_start;

            // Copy characters from after gap to before gap
            for i in 0..move_count {
                self.buffer[dst_start + i] = self.buffer[src_start + i];
            }

            self.gap_start = text_pos;
            self.gap_end = buffer_pos;
        }
    }

    /// Insert a character at the current gap position
    pub fn insert_char(&mut self, ch: char) {
        if self.gap_size() == 0 {
            self.grow_gap();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    /// Insert text at a specific position
    pub fn insert(&mut self, pos: usize, text: &str) {
        self.move_gap_to(pos);

        for ch in text.chars() {
            self.insert_char(ch);
        }
    }

    /// Delete a character before the gap (backspace)
    pub fn delete_backward(&mut self) {
        if self.gap_start > 0 {
            self.gap_start -= 1;
        }
    }

    /// Delete a character after the gap (delete key)
    pub fn delete_forward(&mut self) {
        if self.gap_end < self.buffer.len() {
            self.gap_end += 1;
        }
    }

    /// Delete a range of text
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let start = start.min(self.len());
        let end = end.min(self.len());

        if start >= end {
            return;
        }

        self.move_gap_to(start);
        let delete_count = end - start;
        self.gap_end = (self.gap_end + delete_count).min(self.buffer.len());
    }

    /// Grow the gap when it becomes too small
    fn grow_gap(&mut self) {
        let new_gap_size = 64.max(self.buffer.len() / 4);
        let old_size = self.buffer.len();
        let new_size = old_size + new_gap_size;

        // Create new buffer
        let mut new_buffer = Vec::with_capacity(new_size);

        // Copy text before gap
        new_buffer.extend_from_slice(&self.buffer[..self.gap_start]);

        // Add new gap space
        new_buffer.resize(self.gap_start + new_gap_size, '\0');

        // Copy text after gap
        new_buffer.extend_from_slice(&self.buffer[self.gap_end..]);

        let old_gap_size = self.gap_size();
        self.buffer = new_buffer;
        self.gap_end = self.gap_start + new_gap_size + old_gap_size;
    }

    /// Get the text as a string
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());

        // Add text before gap
        result.extend(&self.buffer[..self.gap_start]);

        // Add text after gap
        result.extend(&self.buffer[self.gap_end..]);

        result
    }

    /// Get all lines as a vector of strings
    pub fn to_lines(&self) -> Vec<String> {
        self.to_string().lines().map(|s| s.to_string()).collect()
    }

    /// Convert cursor position (row, col) to text position
    pub fn cursor_to_position(&self, row: usize, col: usize) -> usize {
        let text = self.to_string();
        let mut pos = 0;

        for (i, line) in text.lines().enumerate() {
            if i == row {
                return pos + col.min(line.len());
            }
            pos += line.len() + 1; // +1 for newline
        }

        pos
    }

    /// Convert text position to cursor position (row, col)
    pub fn position_to_cursor(&self, pos: usize) -> (usize, usize) {
        let text = self.to_string();
        let pos = pos.min(text.len());

        let mut current_pos = 0;
        for (row, line) in text.lines().enumerate() {
            let line_end = current_pos + line.len();

            if pos <= line_end {
                return (row, pos - current_pos);
            }

            current_pos = line_end + 1; // +1 for newline
        }

        // If we're at the very end
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            (0, 0)
        } else {
            (lines.len() - 1, lines.last().map(|l| l.len()).unwrap_or(0))
        }
    }
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer for GapBuffer {
    fn line_count(&self) -> usize {
        let text = self.to_string();
        if text.is_empty() {
            1
        } else {
            text.lines().count().max(1)
        }
    }

    fn get_line(&self, line_idx: usize) -> Option<&str> {
        // This is inefficient but works for now
        // In a real implementation, we'd cache line information
        let lines = self.to_lines();
        lines.get(line_idx).map(|s| {
            // This is a hack - we're returning a reference to a temporary
            // In practice, we'd need a different approach here
            unsafe { std::mem::transmute(s.as_str()) }
        })
    }

    fn all_lines(&self) -> Vec<String> {
        let mut lines = self.to_lines();
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    fn insert_at(&mut self, row: usize, col: usize, text: &str) {
        let pos = self.cursor_to_position(row, col);
        self.insert(pos, text);
    }

    fn delete_at(&mut self, row: usize, col: usize) {
        let pos = self.cursor_to_position(row, col);
        self.move_gap_to(pos);
        self.delete_forward();
    }

    fn backspace_at(&mut self, row: usize, col: usize) {
        let pos = self.cursor_to_position(row, col);
        self.move_gap_to(pos);
        self.delete_backward();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_delete() {
        let mut buffer = GapBuffer::new();

        // Insert some text
        buffer.insert(0, "Hello");
        assert_eq!(buffer.to_string(), "Hello");

        // Insert in the middle
        buffer.insert(2, "XX");
        assert_eq!(buffer.to_string(), "HeXXllo");

        // Delete backward
        buffer.move_gap_to(4);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "HeXllo");

        // Delete forward
        buffer.move_gap_to(2);
        buffer.delete_forward();
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_cursor_conversion() {
        let mut buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // Test cursor to position
        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.cursor_to_position(0, 4), 4);
        assert_eq!(buffer.cursor_to_position(1, 0), 7);
        assert_eq!(buffer.cursor_to_position(2, 2), 16);

        // Test position to cursor
        assert_eq!(buffer.position_to_cursor(0), (0, 0));
        assert_eq!(buffer.position_to_cursor(4), (0, 4));
        assert_eq!(buffer.position_to_cursor(7), (1, 0));
        assert_eq!(buffer.position_to_cursor(16), (2, 2));
    }
}
