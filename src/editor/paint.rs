use super::*;

impl Editor {
    pub fn paint_editor_background(&mut self, window: &mut Window, bounds: Bounds<Pixels>) {
        let bg_color: Hsla = self.config.editor_bg_color.into();

        if bg_color.is_opaque() {
            let editor_bounds = Bounds {
                origin: point(bounds.origin.x + self.config.gutter_width, bounds.origin.y),
                size: size(
                    bounds.size.width - self.config.gutter_width,
                    bounds.size.height,
                ),
            };
            window.paint_quad(PaintQuad {
                bounds: editor_bounds,
                corner_radii: (0.0).into(),
                background: self.config.editor_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    pub fn paint_gutter_background(&mut self, window: &mut Window, bounds: Bounds<Pixels>) {
        let bg_color: Hsla = self.config.gutter_bg_color.into();

        if bg_color.is_opaque() {
            let gutter_bounds = Bounds {
                origin: bounds.origin,
                size: size(self.config.gutter_width, bounds.size.height),
            };
            window.paint_quad(PaintQuad {
                bounds: gutter_bounds,
                corner_radii: (0.0).into(),
                background: self.config.gutter_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    pub fn paint_active_line_background(&mut self, window: &mut Window, bounds: Bounds<Pixels>) {
        let bg_color: Hsla = self.config.active_line_bg_color.into();

        if bg_color.is_opaque() {
            let active_line_bounds = self.line_bounds(self.cursor_position.row, bounds);
            window.paint_quad(PaintQuad {
                bounds: active_line_bounds,
                corner_radii: (0.0).into(),
                background: self.config.active_line_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    pub fn paint_lines(&mut self, cx: &mut App, window: &mut Window, bounds: Bounds<Pixels>) {
        let lines = self.buffer.all_lines();
        for (i, line) in lines.iter().enumerate() {
            let line_bounds = self.line_bounds(i, bounds);
            self.paint_line_number(cx, window, i + 1, line_bounds);
            self.paint_line_content(cx, window, line, line_bounds);
        }
    }

    pub fn paint_line_number(
        &mut self,
        cx: &mut App,
        window: &mut Window,
        line_number: usize,
        line_bounds: Bounds<Pixels>,
    ) {
        let line_number_str = SharedString::new(line_number.to_string());
        let line_number_len = line_number_str.len();
        let gutter_padding = px(10.0);
        let line_number_x =
            line_bounds.origin.x + self.config.gutter_width - gutter_padding - px(20.0);

        let shaped_line_number = window.text_system().shape_line(
            line_number_str,
            self.config.font_size,
            &[TextRun {
                len: line_number_len,
                font: Font {
                    family: self.config.font_family.clone(),
                    features: Default::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: Default::default(),
                },
                color: self.config.line_number_color.into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );

        shaped_line_number
            .paint(
                point(line_number_x, line_bounds.origin.y),
                self.config.line_height,
                window,
                cx,
            )
            .log_err();
    }

    pub fn paint_line_content(
        &mut self,
        cx: &mut App,
        window: &mut Window,
        line: impl Into<SharedString>,
        line_bounds: Bounds<Pixels>,
    ) {
        let gutter_padding = px(10.0);
        let text_x = line_bounds.origin.x + self.config.gutter_width + gutter_padding;
        let line = line.into();

        let shaped_line = window.text_system().shape_line(
            line.clone(),
            self.config.font_size,
            &[TextRun {
                len: line.len(),
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

        shaped_line
            .paint(
                point(text_x, line_bounds.origin.y),
                self.config.line_height,
                window,
                cx,
            )
            .log_err();
    }

    pub fn paint_cursor(&mut self, window: &mut Window, bounds: Bounds<Pixels>) {
        let cursor_pos = self.cursor_position_px(bounds, window);
        let cursor_bounds = Bounds {
            origin: cursor_pos,
            size: size(px(2.0), self.config.line_height),
        };
        window.paint_quad(PaintQuad {
            bounds: cursor_bounds,
            corner_radii: (0.0).into(),
            background: rgb(0xffffff).into(),
            border_color: transparent_black(),
            border_widths: (0.0).into(),
            border_style: BorderStyle::Solid,
        });
    }
}
