use crate::syntax_highlighter::SyntaxHighlighter;
use crate::text_buffer::{SimpleBuffer, TextBuffer};
use gpui::*;

#[derive(Clone)]
pub struct EditorConfig {
    pub line_height: Pixels,
    pub font_size: Pixels,
    pub gutter_width: Pixels,
    pub gutter_padding: Pixels,
    pub text_color: Rgba,
    pub line_number_color: Rgba,
    pub gutter_bg_color: Rgba,
    pub editor_bg_color: Rgba,
    pub active_line_bg_color: Rgba,
    pub font_family: SharedString,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            line_height: px(20.0),
            font_size: px(14.0),
            gutter_width: px(50.0),
            gutter_padding: px(10.0),
            text_color: rgb(0xcccccc),
            line_number_color: rgb(0x666666),
            gutter_bg_color: rgb(0x252525),
            editor_bg_color: rgb(0x1e1e1e),
            active_line_bg_color: rgb(0x2a2a2a),
            font_family: "Monaco".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone)]
pub struct Editor {
    id: ElementId,
    buffer: SimpleBuffer,
    config: EditorConfig,
    cursor_position: CursorPosition,
    goal_column: Option<usize>,
    selection_anchor: Option<CursorPosition>,
    syntax_highlighter: SyntaxHighlighter,
    language: String,
    current_theme: String,
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        let syntax_highlighter = SyntaxHighlighter::new();

        // Auto-detect language from content
        let full_text = lines.join("\n");
        let language = syntax_highlighter
            .detect_language(&full_text, Some("rs"))
            .unwrap_or_else(|| "Rust".to_string());

        Self {
            id,
            buffer: SimpleBuffer::new(lines),
            config: EditorConfig::default(),
            cursor_position: CursorPosition { row: 0, col: 0 },
            goal_column: None,
            selection_anchor: None,
            syntax_highlighter,
            language,
            current_theme: String::new(),
        }
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn config(&self) -> &EditorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut EditorConfig {
        &mut self.config
    }

    pub fn set_config(&mut self, config: EditorConfig) {
        self.config = config;
    }

    pub fn cursor_position(&self) -> CursorPosition {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: CursorPosition) {
        self.cursor_position = position;
        // Reset goal column when cursor position is explicitly set
        self.goal_column = None;
    }

    pub fn get_cursor_position(&self) -> CursorPosition {
        self.cursor_position
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
        // Reset goal column when clearing selection
        self.goal_column = None;
    }

    pub fn get_buffer(&self) -> &SimpleBuffer {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut SimpleBuffer {
        &mut self.buffer
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    pub fn current_theme(&self) -> &str {
        &self.current_theme
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.current_theme = theme.to_string();
        self.syntax_highlighter.set_theme(theme);
        // Update colors from theme
        self.config.editor_bg_color = self.syntax_highlighter.get_theme_background().into();
        self.config.text_color = self.syntax_highlighter.get_theme_foreground().into();
        self.config.gutter_bg_color = self.syntax_highlighter.get_theme_gutter_background().into();
        self.config.active_line_bg_color =
            self.syntax_highlighter.get_theme_line_highlight().into();
    }

    pub fn update_buffer(&mut self, lines: Vec<String>) {
        self.buffer = SimpleBuffer::new(lines);
        // Reset highlighting state to force complete re-highlighting
        self.syntax_highlighter.reset_state();
    }

    /// Update buffer content at a specific line (for future incremental updates)
    pub fn update_line(&mut self, line_index: usize, new_content: String) {
        // Get all lines, update the specific one, then recreate buffer
        let mut lines = self.buffer.all_lines();
        if line_index < lines.len() {
            lines[line_index] = new_content;
            self.buffer = SimpleBuffer::new(lines);
            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(line_index, &self.language);
        }
    }

    /// Get syntax highlighting for a line
    pub fn highlight_line(
        &mut self,
        line: &str,
        line_index: usize,
        font_family: SharedString,
        font_size: f32,
    ) -> Vec<TextRun> {
        self.syntax_highlighter.highlight_line(
            line,
            &self.language,
            line_index,
            font_family,
            font_size,
        )
    }

    // Movement methods
    pub fn move_left(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        // Reset goal column when moving horizontally
        self.goal_column = None;

        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
        } else if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            self.cursor_position.col = self.buffer.line_len(self.cursor_position.row);
        }
    }

    pub fn move_right(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        // Reset goal column when moving horizontally
        self.goal_column = None;

        let current_line_len = self.buffer.line_len(self.cursor_position.row);

        if self.cursor_position.col < current_line_len {
            self.cursor_position.col += 1;
        } else if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Move to start of next line
            self.cursor_position.row += 1;
            self.cursor_position.col = 0;
        }
    }

    pub fn move_up(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        if self.cursor_position.row > 0 {
            // Set goal column if not already set
            if self.goal_column.is_none() {
                self.goal_column = Some(self.cursor_position.col);
            }

            self.cursor_position.row -= 1;

            // Try to use goal column, but clamp to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self
                .goal_column
                .unwrap_or(self.cursor_position.col)
                .min(line_len);
        }
    }

    pub fn move_down(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Set goal column if not already set
            if self.goal_column.is_none() {
                self.goal_column = Some(self.cursor_position.col);
            }

            self.cursor_position.row += 1;

            // Try to use goal column, but clamp to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self
                .goal_column
                .unwrap_or(self.cursor_position.col)
                .min(line_len);
        }
    }

    pub fn select_all(&mut self) {
        // Reset goal column when selecting all
        self.goal_column = None;

        // Set anchor at beginning
        self.selection_anchor = Some(CursorPosition { row: 0, col: 0 });

        // Move cursor to end
        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_row);
        self.cursor_position = CursorPosition {
            row: last_row,
            col: last_col,
        };
    }

    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }

    pub fn get_selection_range(&self) -> Option<(CursorPosition, CursorPosition)> {
        self.selection_anchor.map(|anchor| {
            // Return (start, end) positions in document order
            if anchor.row < self.cursor_position.row
                || (anchor.row == self.cursor_position.row && anchor.col < self.cursor_position.col)
            {
                (anchor, self.cursor_position)
            } else {
                (self.cursor_position, anchor)
            }
        })
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selection_range() {
            // Get all lines
            let mut lines = self.buffer.all_lines();

            if start.row == end.row {
                // Selection within a single line
                let line = &mut lines[start.row];
                let new_line = format!(
                    "{}{}",
                    &line[..start.col.min(line.len())],
                    &line[end.col.min(line.len())..]
                );
                lines[start.row] = new_line;
            } else {
                // Selection spans multiple lines
                let first_line = &lines[start.row];
                let last_line = &lines[end.row];
                let new_line = format!(
                    "{}{}",
                    &first_line[..start.col.min(first_line.len())],
                    &last_line[end.col.min(last_line.len())..]
                );

                // Remove lines in between and replace first line
                lines.splice(start.row..=end.row, vec![new_line]);
            }

            // Update buffer and cursor
            self.buffer = SimpleBuffer::new(lines);
            self.cursor_position = start;
            self.selection_anchor = None;
            self.goal_column = None;

            // Reset highlighting state from the changed line onward
            self.syntax_highlighter
                .clear_state_from_line(start.row, &self.language);

            true
        } else {
            false
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some((start, end)) = self.get_selection_range() {
            let mut selected_text = String::new();
            let lines = self.buffer.all_lines();

            if start.row == end.row {
                // Selection within single line
                let line = &lines[start.row];
                selected_text.push_str(&line[start.col.min(line.len())..end.col.min(line.len())]);
            } else {
                // Selection spans multiple lines
                for (i, line) in lines[start.row..=end.row].iter().enumerate() {
                    let row = start.row + i;
                    if row == start.row {
                        // First line: from start.col to end
                        selected_text.push_str(&line[start.col.min(line.len())..]);
                        selected_text.push('\n');
                    } else if row == end.row {
                        // Last line: from beginning to end.col
                        selected_text.push_str(&line[..end.col.min(line.len())]);
                    } else {
                        // Middle lines: entire line
                        selected_text.push_str(line);
                        selected_text.push('\n');
                    }
                }
            }

            selected_text
        } else {
            String::new()
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        // Delete selection first if there is one
        self.delete_selection();

        let mut lines = self.buffer.all_lines();
        let line = &mut lines[self.cursor_position.row];
        let insert_pos = self.cursor_position.col.min(line.len());
        line.insert(insert_pos, ch);

        self.buffer = SimpleBuffer::new(lines);
        self.cursor_position.col += 1;
        self.goal_column = None;

        // Clear highlighting state from this line onward
        self.syntax_highlighter
            .clear_state_from_line(self.cursor_position.row, &self.language);
    }

    pub fn insert_newline(&mut self) {
        // Delete selection first if there is one
        self.delete_selection();

        let mut lines = self.buffer.all_lines();
        let current_line = lines[self.cursor_position.row].clone();
        let (before, after) =
            current_line.split_at(self.cursor_position.col.min(current_line.len()));

        lines[self.cursor_position.row] = before.to_string();
        lines.insert(self.cursor_position.row + 1, after.to_string());

        self.buffer = SimpleBuffer::new(lines);
        self.cursor_position.row += 1;
        self.cursor_position.col = 0;
        self.goal_column = None;

        // Clear highlighting state from this line onward
        self.syntax_highlighter
            .clear_state_from_line(self.cursor_position.row.saturating_sub(1), &self.language);
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor_position.col > 0 {
            // Delete character before cursor
            let mut lines = self.buffer.all_lines();
            let line = &mut lines[self.cursor_position.row];
            if self.cursor_position.col <= line.len() {
                line.remove(self.cursor_position.col - 1);
            }
            self.buffer = SimpleBuffer::new(lines);
            self.cursor_position.col -= 1;

            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        } else if self.cursor_position.row > 0 {
            // Join with previous line
            let mut lines = self.buffer.all_lines();
            let current_line = lines.remove(self.cursor_position.row);
            let prev_line_len = lines[self.cursor_position.row - 1].len();
            lines[self.cursor_position.row - 1].push_str(&current_line);

            self.buffer = SimpleBuffer::new(lines);
            self.cursor_position.row -= 1;
            self.cursor_position.col = prev_line_len;

            // Clear highlighting state from the previous line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        }

        self.goal_column = None;
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }

        let lines = self.buffer.all_lines();
        let current_line_len = self.buffer.line_len(self.cursor_position.row);

        if self.cursor_position.col < current_line_len {
            // Delete character at cursor
            let mut lines = lines;
            let line = &mut lines[self.cursor_position.row];
            if self.cursor_position.col < line.len() {
                line.remove(self.cursor_position.col);
            }
            self.buffer = SimpleBuffer::new(lines);

            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        } else if self.cursor_position.row < self.buffer.line_count() - 1 {
            // Join with next line
            let mut lines = lines;
            let next_line = lines.remove(self.cursor_position.row + 1);
            lines[self.cursor_position.row].push_str(&next_line);
            self.buffer = SimpleBuffer::new(lines);

            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        }

        self.goal_column = None;
    }
}
