use gpui::*;
use gpui_util::ResultExt;

pub struct Element {
    id: ElementId,
    lines: Vec<String>,
}

impl Element {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        Self { id, lines }
    }
}

impl gpui::Element for Element {
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
        window.paint_quad(PaintQuad {
            bounds,
            corner_radii: (0.0).into(),
            background: rgb(0x1e1e1e).into(),
            border_color: transparent_black(),
            border_widths: (0.0).into(),
            border_style: BorderStyle::Solid,
        });

        let line_height = px(20.0);
        let font_size = px(14.0);
        let text_color = rgb(0xcccccc);

        for (i, line) in self.lines.iter().enumerate() {
            let y = bounds.origin.y + line_height * (i as f32 + 0.75);
            let x = bounds.origin.x + px(10.0);

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
                .paint(point(x, y), line_height, window, cx)
                .log_err();
        }
    }
}

impl IntoElement for Element {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
