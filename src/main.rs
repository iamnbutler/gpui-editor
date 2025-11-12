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
                "use std::collections::HashMap;".to_string(),
                "".to_string(),
                "fn main() {".to_string(),
                "    println!(\"Hello, world!\");".to_string(),
                "    ".to_string(),
                "    // Create some variables".to_string(),
                "    let x = 42;".to_string(),
                "    let name = \"Alice\";".to_string(),
                "    let mut count = 0;".to_string(),
                "    ".to_string(),
                "    // Loop example".to_string(),
                "    for i in 0..10 {".to_string(),
                "        println!(\"Count: {}\", i);".to_string(),
                "        count += 1;".to_string(),
                "    }".to_string(),
                "    ".to_string(),
                "    // HashMap example".to_string(),
                "    let mut scores = HashMap::new();".to_string(),
                "    scores.insert(\"Blue\", 10);".to_string(),
                "    scores.insert(\"Yellow\", 50);".to_string(),
                "    ".to_string(),
                "    println!(\"Final count: {}\", count);".to_string(),
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
