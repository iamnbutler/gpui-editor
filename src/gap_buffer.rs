use crate::text_buffer::TextBuffer;

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
        Self {
            buffer: vec!['\0'; initial_capacity],
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

        let mut buffer = vec!['\0'; total_size];

        // Copy text to beginning of buffer
        for (i, &ch) in chars.iter().enumerate() {
            buffer[i] = ch;
        }

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

    /// Move the gap to a specific text position
    pub fn move_gap_to(&mut self, text_pos: usize) {
        let text_pos = text_pos.min(self.len());

        if text_pos == self.gap_start {
            return; // Gap is already at the right position
        }

        if text_pos < self.gap_start {
            // Move gap left
            let move_count = self.gap_start - text_pos;

            // Move characters from before gap to after gap
            // When moving left, destination is to the right of source, so copy right-to-left
            // to avoid overwriting data we haven't read yet
            for i in (0..move_count).rev() {
                let src = text_pos + i;
                let dst = self.gap_end - move_count + i;
                self.buffer[dst] = self.buffer[src];
            }

            self.gap_end -= move_count;
            self.gap_start -= move_count;
        } else {
            // Move gap right
            // text_pos is in the region after the gap, so we need to account for gap size
            let buffer_pos = text_pos + self.gap_size();
            let move_count = buffer_pos - self.gap_end;

            // Move characters from after gap to before gap
            for i in 0..move_count {
                self.buffer[self.gap_start + i] = self.buffer[self.gap_end + i];
            }

            self.gap_start += move_count;
            self.gap_end += move_count;
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

        // Expand gap to cover the deletion range
        let delete_count = end - start;
        let new_gap_end = (self.gap_end + delete_count).min(self.buffer.len());
        self.gap_end = new_gap_end;
    }

    /// Grow the gap when it becomes too small
    fn grow_gap(&mut self) {
        let new_gap_size = 64.max(self.buffer.len() / 4);
        let old_size = self.buffer.len();
        let new_size = old_size + new_gap_size;

        // Create new buffer with more space
        let mut new_buffer = vec!['\0'; new_size];

        // Copy text before gap
        for i in 0..self.gap_start {
            new_buffer[i] = self.buffer[i];
        }

        // Copy text after gap to the new position
        let text_after_gap = old_size - self.gap_end;
        for i in 0..text_after_gap {
            new_buffer[self.gap_start + new_gap_size + i] = self.buffer[self.gap_end + i];
        }

        self.buffer = new_buffer;
        self.gap_end = self.gap_start + new_gap_size;
    }

    /// Get the text as a string
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());

        // Add text before gap
        for i in 0..self.gap_start {
            result.push(self.buffer[i]);
        }

        // Add text after gap
        for i in self.gap_end..self.buffer.len() {
            result.push(self.buffer[i]);
        }

        result
    }

    /// Get all lines as a vector of strings
    pub fn to_lines(&self) -> Vec<String> {
        let text = self.to_string();
        if text.is_empty() {
            vec![String::new()]
        } else {
            // Split by newline, preserving empty lines at the end
            let mut lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();

            // If text doesn't end with newline, we're done
            // If it does end with newline, split will have added the empty line already

            // Ensure at least one line
            if lines.is_empty() {
                lines.push(String::new());
            }

            lines
        }
    }

    /// Convert cursor position (row, col) to text position
    pub fn cursor_to_position(&self, row: usize, col: usize) -> usize {
        let text = self.to_string();
        let mut pos = 0;

        let lines: Vec<&str> = text.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            if i == row {
                return pos + col.min(line.len());
            }
            pos += line.len();
            if i < lines.len() - 1 {
                pos += 1; // +1 for newline (except on last line)
            }
        }

        pos
    }

    /// Convert text position to cursor position (row, col)
    pub fn position_to_cursor(&self, pos: usize) -> (usize, usize) {
        let text = self.to_string();
        let pos = pos.min(text.len());

        let mut current_pos = 0;
        let lines: Vec<&str> = text.split('\n').collect();

        for (row, line) in lines.iter().enumerate() {
            let line_end = current_pos + line.len();

            if pos <= line_end {
                return (row, pos - current_pos);
            }

            current_pos = line_end;
            if row < lines.len() - 1 {
                current_pos += 1; // +1 for newline (except on last line)
            }
        }

        // If we're at the very end
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
            text.split('\n').count().max(1)
        }
    }

    fn get_line(&self, line_idx: usize) -> Option<&str> {
        todo!("impl get_line")
    }

    fn all_lines(&self) -> Vec<String> {
        self.to_lines()
    }

    fn line_len(&self, line_idx: usize) -> usize {
        let lines = self.all_lines();
        lines.get(line_idx).map(|s| s.len()).unwrap_or(0)
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
    fn test_line_splitting_with_newlines() {
        let mut buffer = GapBuffer::from_text("foo\nbar\nbaz");
        let lines = buffer.all_lines();
        assert_eq!(lines, vec!["foo", "bar", "baz"]);

        // Test with trailing newline
        let mut buffer = GapBuffer::from_text("foo\n");
        let lines = buffer.all_lines();
        assert_eq!(
            lines,
            vec!["foo", ""],
            "Should have empty line after trailing newline"
        );

        // Test with multiple trailing newlines
        let mut buffer = GapBuffer::from_text("foo\n\n");
        let lines = buffer.all_lines();
        assert_eq!(
            lines,
            vec!["foo", "", ""],
            "Should have two empty lines after two newlines"
        );

        // Test empty buffer
        let mut buffer = GapBuffer::from_text("");
        let lines = buffer.all_lines();
        assert_eq!(lines, vec![""], "Empty buffer should have one empty line");
    }

    #[test]
    fn test_cursor_position_conversion() {
        let buffer = GapBuffer::from_text("foo\nbar\nbaz");

        // Test cursor to position
        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.cursor_to_position(0, 3), 3);
        assert_eq!(buffer.cursor_to_position(1, 0), 4);
        assert_eq!(buffer.cursor_to_position(1, 3), 7);
        assert_eq!(buffer.cursor_to_position(2, 0), 8);

        // Test position to cursor
        assert_eq!(buffer.position_to_cursor(0), (0, 0));
        assert_eq!(buffer.position_to_cursor(3), (0, 3));
        assert_eq!(buffer.position_to_cursor(4), (1, 0));
        assert_eq!(buffer.position_to_cursor(7), (1, 3));
        assert_eq!(buffer.position_to_cursor(8), (2, 0));
    }

    #[test]
    fn test_newline_insertion() {
        let mut buffer = GapBuffer::from_text("foo\nbar");

        // Insert newline in middle of first line
        buffer.insert(2, "\n");
        assert_eq!(buffer.to_string(), "fo\no\nbar");
        let lines = buffer.all_lines();
        assert_eq!(lines, vec!["fo", "o", "bar"]);

        // Insert newline at end
        let mut buffer = GapBuffer::from_text("foo");
        buffer.insert(3, "\n");
        assert_eq!(buffer.to_string(), "foo\n");
        let lines = buffer.all_lines();
        assert_eq!(
            lines,
            vec!["foo", ""],
            "Should have empty line after newline at end"
        );
    }
    use crate::text_buffer::TextBuffer;

    #[test]
    fn test_new_empty_buffer() {
        let buffer = GapBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.to_string(), "");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.all_lines(), vec![""]);
    }

    #[test]
    fn test_from_text() {
        let buffer = GapBuffer::from_text("Hello, World!");
        assert_eq!(buffer.to_string(), "Hello, World!");
        assert_eq!(buffer.len(), 13);
        assert_eq!(buffer.line_count(), 1);

        let buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.to_string(), "Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_from_lines() {
        let lines = vec![
            "First line".to_string(),
            "Second line".to_string(),
            "Third line".to_string(),
        ];
        let buffer = GapBuffer::from_lines(lines);
        assert_eq!(buffer.to_string(), "First line\nSecond line\nThird line");
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = GapBuffer::new();

        buffer.insert_char('H');
        assert_eq!(buffer.to_string(), "H");

        buffer.insert_char('i');
        assert_eq!(buffer.to_string(), "Hi");

        // Move gap and insert
        buffer.move_gap_to(1);
        buffer.insert_char('e');
        assert_eq!(buffer.to_string(), "Hei");
    }

    #[test]
    fn test_insert_text_various_positions() {
        let mut buffer = GapBuffer::new();

        // Insert at beginning (empty buffer)
        buffer.insert(0, "Hello");
        assert_eq!(buffer.to_string(), "Hello");

        // Insert at end
        buffer.insert(5, " World");
        assert_eq!(buffer.to_string(), "Hello World");

        // Insert in middle
        buffer.insert(5, " Beautiful");
        assert_eq!(buffer.to_string(), "Hello Beautiful World");

        // Insert at beginning
        buffer.insert(0, "Say: ");
        assert_eq!(buffer.to_string(), "Say: Hello Beautiful World");
    }

    #[test]
    fn test_insert_multiline() {
        let mut buffer = GapBuffer::new();

        buffer.insert(0, "Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.to_string(), "Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);

        // Insert in middle of line 2
        buffer.insert(11, " inserted");
        assert_eq!(buffer.to_string(), "Line 1\nLine inserted 2\nLine 3");

        // Insert newline
        buffer.insert(6, "\nNew Line");
        assert_eq!(
            buffer.to_string(),
            "Line 1\nNew Line\nLine inserted 2\nLine 3"
        );
        assert_eq!(buffer.line_count(), 4);
    }

    #[test]
    fn test_delete_backward() {
        let mut buffer = GapBuffer::from_text("Hello World");

        // Delete from end
        buffer.move_gap_to(11);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "Hello Worl");

        // Delete from middle
        buffer.move_gap_to(6);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "HelloWorl");

        // Delete from beginning (should do nothing)
        buffer.move_gap_to(0);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "HelloWorl");
    }

    #[test]
    fn test_delete_forward() {
        let mut buffer = GapBuffer::from_text("Hello World");

        // Delete from beginning
        buffer.move_gap_to(0);
        buffer.delete_forward();
        assert_eq!(buffer.to_string(), "ello World");

        // Delete from middle
        buffer.move_gap_to(4);
        buffer.delete_forward();
        assert_eq!(buffer.to_string(), "elloWorld");

        // Delete from end (should do nothing)
        buffer.move_gap_to(9);
        buffer.delete_forward();
        assert_eq!(buffer.to_string(), "elloWorld");
    }

    #[test]
    fn test_delete_range() {
        let mut buffer = GapBuffer::from_text("Hello Beautiful World");

        // Delete word in middle
        buffer.delete_range(6, 16);
        assert_eq!(buffer.to_string(), "Hello World");

        // Delete from beginning
        buffer.delete_range(0, 6);
        assert_eq!(buffer.to_string(), "World");

        // Delete to end
        buffer.delete_range(3, 5);
        assert_eq!(buffer.to_string(), "Wor");

        // Delete all
        buffer.delete_range(0, 3);
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_delete_range_edge_cases() {
        let mut buffer = GapBuffer::from_text("Test");

        // Delete with inverted range (should do nothing)
        buffer.delete_range(3, 1);
        assert_eq!(buffer.to_string(), "Test");

        // Delete beyond bounds
        buffer.delete_range(2, 100);
        assert_eq!(buffer.to_string(), "Te");

        // Delete from beyond bounds
        buffer.delete_range(50, 100);
        assert_eq!(buffer.to_string(), "Te");
    }

    #[test]
    fn test_move_gap_to() {
        let mut buffer = GapBuffer::from_text("ABCDEF");

        // Test gap movement doesn't affect content
        for i in 0..=6 {
            buffer.move_gap_to(i);
            assert_eq!(buffer.to_string(), "ABCDEF", "Gap at position {}", i);
        }

        // Test inserting after moving gap
        buffer.move_gap_to(3);
        buffer.insert_char('X');
        assert_eq!(buffer.to_string(), "ABCXDEF");
    }

    #[test]
    fn test_gap_movement_with_operations() {
        let mut buffer = GapBuffer::from_text("123456789");

        // Move gap back and forth with operations
        buffer.move_gap_to(9);
        buffer.insert_char('A');
        assert_eq!(buffer.to_string(), "123456789A");

        buffer.move_gap_to(0);
        buffer.insert_char('B');
        assert_eq!(buffer.to_string(), "B123456789A");

        buffer.move_gap_to(5);
        buffer.insert_char('C');
        assert_eq!(buffer.to_string(), "B1234C56789A");

        buffer.move_gap_to(7);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "B1234C6789A");
    }

    #[test]
    fn test_cursor_to_position() {
        let buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // First line
        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.cursor_to_position(0, 3), 3);
        assert_eq!(buffer.cursor_to_position(0, 6), 6);

        // Second line
        assert_eq!(buffer.cursor_to_position(1, 0), 7);
        assert_eq!(buffer.cursor_to_position(1, 3), 10);
        assert_eq!(buffer.cursor_to_position(1, 6), 13);

        // Third line
        assert_eq!(buffer.cursor_to_position(2, 0), 14);
        assert_eq!(buffer.cursor_to_position(2, 3), 17);
        assert_eq!(buffer.cursor_to_position(2, 6), 20);

        // Beyond line length (should clamp)
        assert_eq!(buffer.cursor_to_position(0, 100), 6);
        assert_eq!(buffer.cursor_to_position(1, 100), 13);
    }

    #[test]
    fn test_position_to_cursor() {
        let buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // First line
        assert_eq!(buffer.position_to_cursor(0), (0, 0));
        assert_eq!(buffer.position_to_cursor(3), (0, 3));
        assert_eq!(buffer.position_to_cursor(6), (0, 6));

        // Second line
        assert_eq!(buffer.position_to_cursor(7), (1, 0));
        assert_eq!(buffer.position_to_cursor(10), (1, 3));
        assert_eq!(buffer.position_to_cursor(13), (1, 6));

        // Third line
        assert_eq!(buffer.position_to_cursor(14), (2, 0));
        assert_eq!(buffer.position_to_cursor(17), (2, 3));
        assert_eq!(buffer.position_to_cursor(20), (2, 6));

        // Beyond text length
        assert_eq!(buffer.position_to_cursor(100), (2, 6));
    }

    #[test]
    fn test_cursor_position_roundtrip() {
        let buffer = GapBuffer::from_text("A\nBB\nCCC\nDDDD");

        for row in 0..4 {
            for col in 0..=row + 1 {
                let pos = buffer.cursor_to_position(row, col);
                let (r, c) = buffer.position_to_cursor(pos);
                assert_eq!(
                    (r, c),
                    (row, col.min(row + 1)),
                    "Roundtrip failed for ({}, {})",
                    row,
                    col
                );
            }
        }
    }

    #[test]
    fn test_to_lines() {
        let buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.to_lines(), vec!["Line 1", "Line 2", "Line 3"]);

        let buffer = GapBuffer::from_text("Single line");
        assert_eq!(buffer.to_lines(), vec!["Single line"]);

        let buffer = GapBuffer::from_text("");
        assert_eq!(buffer.to_lines(), vec![""]);

        let buffer = GapBuffer::from_text("Line with\n\nempty line");
        assert_eq!(buffer.to_lines(), vec!["Line with", "", "empty line"]);
    }

    #[test]
    fn test_line_count() {
        assert_eq!(GapBuffer::from_text("").line_count(), 1);
        assert_eq!(GapBuffer::from_text("One line").line_count(), 1);
        assert_eq!(GapBuffer::from_text("Line 1\nLine 2").line_count(), 2);
        assert_eq!(
            GapBuffer::from_text("Line 1\nLine 2\nLine 3").line_count(),
            3
        );
        assert_eq!(GapBuffer::from_text("Line 1\n\nLine 3").line_count(), 3);
    }

    // todo: impl get_line properly
    // #[test]
    // fn test_get_line() {
    //     let buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

    //     // Note: get_line returns None for GapBuffer implementation
    //     // Use all_lines() instead
    //     assert_eq!(buffer.get_line(0), None);
    //     assert_eq!(buffer.get_line(1), None);
    //     assert_eq!(buffer.get_line(2), None);

    //     // Verify data is accessible via all_lines()
    //     let lines = buffer.all_lines();
    //     assert_eq!(lines[0], "Line 1");
    //     assert_eq!(lines[1], "Line 2");
    //     assert_eq!(lines[2], "Line 3");
    // }

    #[test]
    fn test_line_len() {
        let buffer = GapBuffer::from_text("Short\nMedium line\nA very long line here");

        assert_eq!(buffer.line_len(0), 5);
        assert_eq!(buffer.line_len(1), 11);
        assert_eq!(buffer.line_len(2), 21);
        assert_eq!(buffer.line_len(3), 0); // Beyond line count
    }

    #[test]
    fn test_insert_at() {
        let mut buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // Insert in middle of line
        buffer.insert_at(1, 5, "X");
        assert_eq!(buffer.all_lines()[1], "Line X2");

        // Insert at beginning of line
        buffer.insert_at(0, 0, "Start: ");
        assert_eq!(buffer.all_lines()[0], "Start: Line 1");

        // Insert at end of line
        buffer.insert_at(2, 6, " End");
        assert_eq!(buffer.all_lines()[2], "Line 3 End");
    }

    #[test]
    fn test_delete_at() {
        let mut buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // Delete in middle of line
        buffer.delete_at(1, 5);
        assert_eq!(buffer.all_lines()[1], "Line ");

        // Delete at end of line (merges with next)
        buffer.delete_at(0, 6);
        assert_eq!(buffer.to_lines(), vec!["Line 1Line ", "Line 3"]);
    }

    #[test]
    fn test_backspace_at() {
        let mut buffer = GapBuffer::from_text("Line 1\nLine 2\nLine 3");

        // Backspace in middle of line
        buffer.backspace_at(1, 5);
        assert_eq!(buffer.all_lines()[1], "Line2");

        // Backspace at beginning of line (merges with previous)
        buffer.backspace_at(2, 0);
        assert_eq!(buffer.to_lines(), vec!["Line 1", "Line2Line 3"]);

        // Backspace at beginning of first line (should do nothing)
        buffer.backspace_at(0, 0);
        assert_eq!(buffer.to_lines(), vec!["Line 1", "Line2Line 3"]);
    }

    #[test]
    fn test_grow_gap() {
        let mut buffer = GapBuffer::new();

        // Test simple growth
        let text = "a".repeat(100);
        buffer.insert(0, &text);
        assert_eq!(buffer.to_string(), text);
        assert_eq!(buffer.len(), 100);
    }

    #[test]
    fn test_gap_movement_preserves_text() {
        let mut buffer = GapBuffer::new();

        // Insert ABC at beginning
        buffer.insert(0, "ABC");
        assert_eq!(buffer.to_string(), "ABC");

        // Insert DEF at end
        buffer.insert(3, "DEF");
        assert_eq!(buffer.to_string(), "ABCDEF");

        // Move gap to middle and insert
        buffer.insert(3, "123");
        assert_eq!(buffer.to_string(), "ABC123DEF");

        // Move gap to beginning and insert
        buffer.insert(0, ">");
        assert_eq!(buffer.to_string(), ">ABC123DEF");

        // Move gap to end and insert
        buffer.insert(10, "<");
        assert_eq!(buffer.to_string(), ">ABC123DEF<");
    }

    #[test]
    fn test_large_buffer_with_gap_movement() {
        let mut buffer = GapBuffer::new();

        // Create a scenario similar to the failing case but simpler
        // Insert 70 a's (exceeds initial 64 capacity)
        buffer.insert(0, &"a".repeat(70));
        assert_eq!(buffer.to_string(), "a".repeat(70));

        // Insert 50 b's at the end
        buffer.insert(70, &"b".repeat(50));
        let expected = format!("{}{}", "a".repeat(70), "b".repeat(50));
        assert_eq!(buffer.to_string(), expected);

        // Now move gap to middle and insert - this is where it was failing
        buffer.insert(60, "XXX");
        let expected = format!("{}XXX{}{}", "a".repeat(60), "a".repeat(10), "b".repeat(50));
        assert_eq!(buffer.to_string(), expected);
    }

    #[test]
    fn test_empty_buffer_operations() {
        let mut buffer = GapBuffer::new();

        // Operations on empty buffer
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "");

        buffer.delete_forward();
        assert_eq!(buffer.to_string(), "");

        buffer.delete_range(0, 10);
        assert_eq!(buffer.to_string(), "");

        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.position_to_cursor(0), (0, 0));
    }

    #[test]
    fn test_newline_handling() {
        let mut buffer = GapBuffer::new();

        // Insert text with newlines
        buffer.insert(0, "A\nB");
        assert_eq!(buffer.line_count(), 2);
        let lines = buffer.all_lines();
        assert_eq!(lines[0], "A");
        assert_eq!(lines[1], "B");

        // Insert newline in middle
        buffer.insert(1, "\n");
        assert_eq!(buffer.line_count(), 3);
        let lines = buffer.all_lines();
        assert_eq!(lines[0], "A");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "B");

        // Delete newline
        buffer.move_gap_to(2);
        buffer.delete_backward();
        assert_eq!(buffer.line_count(), 2);
        let lines = buffer.all_lines();
        assert_eq!(lines[0], "A");
        assert_eq!(lines[1], "B");
    }

    #[test]
    fn test_stress_random_operations() {
        let mut buffer = GapBuffer::new();
        let test_string = "The quick brown fox jumps over the lazy dog";

        // Build string character by character at random positions
        let chars: Vec<char> = test_string.chars().collect();
        let mut positions = vec![];

        for (i, ch) in chars.iter().enumerate() {
            let pos = i / 2; // Insert roughly in middle as we go
            buffer.insert(pos, &ch.to_string());
            positions.push(pos);
        }

        // The string won't match exactly due to random positions,
        // but should have same length
        assert_eq!(buffer.len(), test_string.len());

        // Delete some characters
        for _ in 0..10 {
            if buffer.len() > 0 {
                buffer.move_gap_to(buffer.len() / 2);
                buffer.delete_forward();
            }
        }

        assert_eq!(buffer.len(), test_string.len() - 10);
    }

    #[test]
    fn test_large_text() {
        let large_text = "Line\n".repeat(1000);
        let mut buffer = GapBuffer::from_text(&large_text);

        assert_eq!(buffer.line_count(), 1001); // 1000 lines + 1 empty line after trailing newline

        // Insert in middle of large text
        buffer.insert(2500, "INSERTED");
        assert!(buffer.to_string().contains("INSERTED"));

        // Delete range in large text
        buffer.delete_range(2500, 2508);
        assert!(!buffer.to_string().contains("INSERTED"));

        // Verify structure is intact
        assert_eq!(buffer.line_count(), 1001); // Still has trailing newline
    }

    #[test]
    fn test_unicode_characters() {
        let mut buffer = GapBuffer::from_text("Hello 世界");
        assert_eq!(buffer.to_string(), "Hello 世界");
        assert_eq!(buffer.len(), 8); // Note: char count, not byte count

        buffer.insert(6, "🦀");
        assert_eq!(buffer.to_string(), "Hello 🦀世界");

        buffer.move_gap_to(7);
        buffer.delete_backward();
        assert_eq!(buffer.to_string(), "Hello 世界");

        // Test with emoji and various unicode
        let unicode_text = "😀😃😄 नमस्ते мир";
        buffer = GapBuffer::from_text(unicode_text);
        assert_eq!(buffer.to_string(), unicode_text);
    }

    #[test]
    fn test_sequential_edits() {
        let mut buffer = GapBuffer::new();

        // Simulate typing
        let text = "Hello World!";
        for ch in text.chars() {
            buffer.insert_char(ch);
        }
        assert_eq!(buffer.to_string(), text);

        // Simulate backspacing
        for _ in 0..6 {
            buffer.delete_backward();
        }
        assert_eq!(buffer.to_string(), "Hello ");

        // Continue typing
        for ch in "Rust!".chars() {
            buffer.insert_char(ch);
        }
        assert_eq!(buffer.to_string(), "Hello Rust!");
    }
}
