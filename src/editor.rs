use gpui::*;
use gpui_util::ResultExt;

pub struct Editor {
    id: ElementId,
    lines: Vec<String>,
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        Self { id, lines }
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
        let line_height = px(20.0);
        let font_size = px(14.0);
        let gutter_width = px(50.0);
        let gutter_padding = px(10.0);
        let text_color = rgb(0xcccccc);
        let line_number_color = rgb(0x666666);
        let gutter_bg_color = transparent_black();
        let editor_bg_color = transparent_black();

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

        for (i, line) in self.lines.iter().enumerate() {
            let y = bounds.origin.y + line_height * (i as f32 + 0.75);

            let line_number = SharedString::new((i + 1).to_string());
            let line_number_len = line_number.len();
            let line_number_x = bounds.origin.x + gutter_width - gutter_padding - px(20.0); // Right-align

            let shaped_line_number = window.text_system().shape_line(
                line_number,
                font_size,
                &[TextRun {
                    len: line_number_len,
                    font: Font {
                        family: "Monaco".into(),
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
                .paint(point(line_number_x, y), line_height, window, cx)
                .log_err();

            let text_x = bounds.origin.x + gutter_width + gutter_padding;

            let shaped_line = window.text_system().shape_line(
                line.into(),
                font_size,
                &[TextRun {
                    len: line.len(),
                    font: Font {
                        family: "Monaco".into(),
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
                .paint(point(text_x, y), line_height, window, cx)
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
