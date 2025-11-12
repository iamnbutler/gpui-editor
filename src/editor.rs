use gpui::*;
use gpui_util::ResultExt;

mod paint;

use crate::text_buffer::{SimpleBuffer, TextBuffer};

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

#[derive(Clone, Copy, Debug)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}

pub struct Editor {
    id: ElementId,
    buffer: SimpleBuffer,
    config: EditorConfig,
    cursor_position: CursorPosition,
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        Self {
            id,
            buffer: SimpleBuffer::new(lines),
            config: EditorConfig::default(),
            cursor_position: CursorPosition { row: 0, col: 0 },
        }
    }

    pub fn config(mut self, config: EditorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn cursor_position(mut self, position: CursorPosition) -> Self {
        self.cursor_position = position;
        self
    }

    pub fn set_cursor_position(&mut self, position: CursorPosition) {
        self.cursor_position = position;
    }

    pub fn move_left(&mut self) {
        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
        } else if self.cursor_position.row > 0 {
            // Move to end of previous line
            self.cursor_position.row -= 1;
            self.cursor_position.col = self.buffer.line_len(self.cursor_position.row);
        }
    }

    pub fn move_right(&mut self) {
        let current_line_len = self.buffer.line_len(self.cursor_position.row);

        if self.cursor_position.col < current_line_len {
            self.cursor_position.col += 1;
        } else if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Move to start of next line
            self.cursor_position.row += 1;
            self.cursor_position.col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            // Clamp column to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self.cursor_position.col.min(line_len);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            self.cursor_position.row += 1;
            // Clamp column to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self.cursor_position.col.min(line_len);
        }
    }

    /// Calculate the y position where a line starts
    fn y_for_line(&self, line_index: usize, bounds: Bounds<Pixels>) -> Pixels {
        bounds.origin.y + self.config.line_height * line_index as f32
    }

    /// Calculate the bounds for a given line (for backgrounds, etc)
    fn line_bounds(&self, line_index: usize, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds {
            origin: point(
                bounds.origin.x + self.config.gutter_width,
                bounds.origin.y + self.config.line_height * line_index as f32,
            ),
            size: size(
                bounds.size.width - self.config.gutter_width,
                self.config.line_height,
            ),
        }
    }

    /// Calculate the pixel position for the cursor
    fn cursor_position_px(&self, bounds: Bounds<Pixels>, window: &Window) -> Point<Pixels> {
        let line_y = self.y_for_line(self.cursor_position.row, bounds);
        let text_x_start = bounds.origin.x + self.config.gutter_width + self.config.gutter_padding;

        // Calculate x position based on column
        let mut x_offset = Pixels::ZERO;
        if let Some(line) = self.buffer.get_line(self.cursor_position.row) {
            if self.cursor_position.col > 0 {
                let text_before_cursor = SharedString::from(
                    line[..self.cursor_position.col.min(line.len())].to_string(),
                );
                let shaped = window.text_system().shape_line(
                    text_before_cursor.clone(),
                    self.config.font_size,
                    &[TextRun {
                        len: text_before_cursor.len(),
                        font: Font {
                            family: self.config.font_family.clone(),
                            features: Default::default(),
                            weight: FontWeight::NORMAL,
                            style: FontStyle::Normal,
                            fallbacks: Default::default(),
                        },
                        color: self.config.text_color.into(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    }],
                    None,
                );
                x_offset = shaped.width;
            }
        }

        point(text_x_start + x_offset, line_y)
    }

    /// Convert a pixel position to a cursor position
    pub fn position_to_cursor(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> CursorPosition {
        // Check if click is in gutter
        if position.x < bounds.origin.x + self.config.gutter_width {
            // Clicking gutter selects whole line
            let line_index =
                ((position.y - bounds.origin.y) / self.config.line_height).floor() as usize;
            let line_index = line_index.min(self.buffer.line_count().saturating_sub(1));
            return CursorPosition {
                row: line_index,
                col: 0, // For now, just position at start of line
            };
        }

        // Calculate which line was clicked
        let line_index =
            ((position.y - bounds.origin.y) / self.config.line_height).floor() as usize;
        let line_index = line_index.min(self.buffer.line_count().saturating_sub(1));

        // Calculate which column was clicked
        let text_x_start = bounds.origin.x + self.config.gutter_width + self.config.gutter_padding;
        let relative_x = (position.x - text_x_start).max(px(0.0));

        // Get the line content
        let col = if let Some(line) = self.buffer.get_line(line_index) {
            if line.is_empty() {
                0
            } else {
                // Binary search to find the column
                let mut left = 0;
                let mut right = line.len();
                let mut best_col = 0;
                let mut best_distance = px(999999.0);

                while left <= right {
                    let mid = (left + right) / 2;

                    // Measure width up to this column
                    let text_before = SharedString::from(line[..mid].to_string());
                    let shaped = window.text_system().shape_line(
                        text_before.clone(),
                        self.config.font_size,
                        &[TextRun {
                            len: text_before.len(),
                            font: Font {
                                family: self.config.font_family.clone(),
                                features: Default::default(),
                                weight: FontWeight::NORMAL,
                                style: FontStyle::Normal,
                                fallbacks: Default::default(),
                            },
                            color: self.config.text_color.into(),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        }],
                        None,
                    );

                    let width = shaped.width;
                    let distance = (width - relative_x).abs();

                    if distance < best_distance {
                        best_distance = distance;
                        best_col = mid;
                    }

                    if width < relative_x {
                        left = mid + 1;
                    } else if width > relative_x && mid > 0 {
                        right = mid - 1;
                    } else {
                        break;
                    }
                }

                // Check if we should position after the last character
                if best_col < line.len() && relative_x > px(0.0) {
                    // Measure the full width including the next character
                    let text_with_next = SharedString::from(line[..best_col + 1].to_string());
                    let shaped_with_next = window.text_system().shape_line(
                        text_with_next.clone(),
                        self.config.font_size,
                        &[TextRun {
                            len: text_with_next.len(),
                            font: Font {
                                family: self.config.font_family.clone(),
                                features: Default::default(),
                                weight: FontWeight::NORMAL,
                                style: FontStyle::Normal,
                                fallbacks: Default::default(),
                            },
                            color: self.config.text_color.into(),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        }],
                        None,
                    );

                    let text_without_next = if best_col > 0 {
                        SharedString::from(line[..best_col].to_string())
                    } else {
                        SharedString::from("")
                    };

                    let shaped_without_next = window.text_system().shape_line(
                        text_without_next.clone(),
                        self.config.font_size,
                        &[TextRun {
                            len: text_without_next.len(),
                            font: Font {
                                family: self.config.font_family.clone(),
                                features: Default::default(),
                                weight: FontWeight::NORMAL,
                                style: FontStyle::Normal,
                                fallbacks: Default::default(),
                            },
                            color: self.config.text_color.into(),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        }],
                        None,
                    );

                    let width_before = shaped_without_next.width;
                    let width_after = shaped_with_next.width;
                    let midpoint = (width_before + width_after) / 2.0;

                    if relative_x > midpoint {
                        best_col + 1
                    } else {
                        best_col
                    }
                } else {
                    best_col
                }
            }
        } else {
            0
        };

        // Clamp column to line length
        let col = col.min(self.buffer.line_len(line_index));

        CursorPosition {
            row: line_index,
            col,
        }
    }
}

impl gpui::Element for Editor {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();

        let layout_id = window.request_layout(style, None, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout_state: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout_state: &mut Self::RequestLayoutState,
        _prepaint_state: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let line_height = self.config.line_height;
        let font_size = self.config.font_size;
        let gutter_width = self.config.gutter_width;
        let gutter_padding = self.config.gutter_padding;
        let text_color = self.config.text_color;
        let line_number_color = self.config.line_number_color;
        let gutter_bg_color = self.config.gutter_bg_color;
        let editor_bg_color = self.config.editor_bg_color;
        let active_line_bg_color = self.config.active_line_bg_color;

        // background
        self.paint_gutter_background(window, bounds);
        self.paint_editor_background(window, bounds);

        // underlay
        self.paint_active_line_background(window, bounds);

        // content
        self.paint_lines(cx, window, bounds);

        // overlay
        self.paint_cursor(window, bounds);
    }
}

impl IntoElement for Editor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
