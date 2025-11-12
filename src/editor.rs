use gpui::*;
use gpui_util::ResultExt;

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
    lines: Vec<String>,
    config: EditorConfig,
    cursor_position: CursorPosition,
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        Self {
            id,
            lines,
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

        let gutter_bounds = Bounds {
            origin: bounds.origin,
            size: size(gutter_width, bounds.size.height),
        };
        window.paint_quad(PaintQuad {
            bounds: gutter_bounds,
            corner_radii: (0.0).into(),
            background: gutter_bg_color.into(),
            border_color: transparent_black(),
            border_widths: (0.0).into(),
            border_style: BorderStyle::Solid,
        });

        let editor_bounds = Bounds {
            origin: point(bounds.origin.x + gutter_width, bounds.origin.y),
            size: size(bounds.size.width - gutter_width, bounds.size.height),
        };
        window.paint_quad(PaintQuad {
            bounds: editor_bounds,
            corner_radii: (0.0).into(),
            background: editor_bg_color.into(),
            border_color: transparent_black(),
            border_widths: (0.0).into(),
            border_style: BorderStyle::Solid,
        });

        // Paint active line background first (before any text)
        let active_line_bounds = self.line_bounds(self.cursor_position.row, bounds);
        window.paint_quad(PaintQuad {
            bounds: active_line_bounds,
            corner_radii: (0.0).into(),
            background: active_line_bg_color.into(),
            border_color: transparent_black(),
            border_widths: (0.0).into(),
            border_style: BorderStyle::Solid,
        });

        // Now paint all text on top
        for (i, line) in self.lines.iter().enumerate() {
            let line_y = self.y_for_line(i, bounds);

            let line_number = SharedString::new((i + 1).to_string());
            let line_number_len = line_number.len();
            let line_number_x = bounds.origin.x + gutter_width - gutter_padding - px(20.0); // Right-align

            let shaped_line_number = window.text_system().shape_line(
                line_number,
                font_size,
                &[TextRun {
                    len: line_number_len,
                    font: Font {
                        family: self.config.font_family.clone(),
                        features: Default::default(),
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Normal,
                        fallbacks: Default::default(),
                    },
                    color: line_number_color.into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );

            shaped_line_number
                .paint(point(line_number_x, line_y), line_height, window, cx)
                .log_err();

            let text_x = bounds.origin.x + gutter_width + gutter_padding;

            let shaped_line = window.text_system().shape_line(
                line.into(),
                font_size,
                &[TextRun {
                    len: line.len(),
                    font: Font {
                        family: self.config.font_family.clone(),
                        features: Default::default(),
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Normal,
                        fallbacks: Default::default(),
                    },
                    color: text_color.into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );

            shaped_line
                .paint(point(text_x, line_y), line_height, window, cx)
                .log_err();
        }
    }
}

impl IntoElement for Editor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
