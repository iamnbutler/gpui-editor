use gpui::*;

mod editor;
use editor::Editor;

struct EditorView {
    lines: Vec<String>,
}

impl EditorView {
    fn new() -> Self {
        Self {
            lines: vec![
                "fn main() {".to_string(),
                "    println!(\"Hello, world!\");".to_string(),
                "    let x = 42;".to_string(),
                "}".to_string(),
            ],
        }
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let editor_element = Editor::new("editor", self.lines.clone());

        editor_element
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Editor Element".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| EditorView::new()),
        )
        .unwrap();

        cx.activate(false);
    });
}
