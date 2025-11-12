use super::*;

impl Editor {
    pub fn paint_gutter_background(&mut self, window: &mut Window, bounds: Bounds<Pixels>) {
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
