use gpui::*;

mod editor;
use editor::{CursorPosition, Editor, EditorConfig};

actions!(editor_view, [MoveUp, MoveDown, MoveLeft, MoveRight]);

struct EditorView {
    focus_handle: FocusHandle,
    lines: Vec<String>,
    cursor_position: CursorPosition,
}

impl EditorView {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            focus_handle,
            cursor_position: CursorPosition { row: 0, col: 0 },
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

    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            // Clamp column to line length
            let line_len = self
                .lines
                .get(self.cursor_position.row)
                .map(|line| line.len())
                .unwrap_or(0);
            self.cursor_position.col = self.cursor_position.col.min(line_len);
            cx.notify();
        }
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor_position.row < self.lines.len().saturating_sub(1) {
            self.cursor_position.row += 1;
            // Clamp column to line length
            let line_len = self
                .lines
                .get(self.cursor_position.row)
                .map(|line| line.len())
                .unwrap_or(0);
            self.cursor_position.col = self.cursor_position.col.min(line_len);
            cx.notify();
        }
    }

    fn move_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
        } else if self.cursor_position.row > 0 {
            // Move to end of previous line
            self.cursor_position.row -= 1;
            self.cursor_position.col = self
                .lines
                .get(self.cursor_position.row)
                .map(|line| line.len())
                .unwrap_or(0);
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        let current_line_len = self
            .lines
            .get(self.cursor_position.row)
            .map(|line| line.len())
            .unwrap_or(0);

        if self.cursor_position.col < current_line_len {
            self.cursor_position.col += 1;
        } else if self.cursor_position.row < self.lines.len().saturating_sub(1) {
            // Move to start of next line
            self.cursor_position.row += 1;
            self.cursor_position.col = 0;
        }
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = EditorConfig {
            line_height: px(22.0),
            font_size: px(15.0),
            gutter_width: px(60.0),
            gutter_padding: px(12.0),
            text_color: rgb(0xdcdcdc),
            line_number_color: rgb(0x858585),
            gutter_bg_color: rgb(0x2a2a2a),
            editor_bg_color: rgb(0x1e1e1e),
            active_line_bg_color: rgb(0x2a2a2a),
            font_family: "JetBrains Mono".into(),
        };

        let editor_element = Editor::new("editor", self.lines.clone())
            .config(config)
            .cursor_position(self.cursor_position);

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::move_left))
            .on_action(cx.listener(Self::move_right))
            .child(editor_element)
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Bind keyboard shortcuts
        cx.bind_keys([
            KeyBinding::new("up", MoveUp, None),
            KeyBinding::new("down", MoveDown, None),
            KeyBinding::new("left", MoveLeft, None),
            KeyBinding::new("right", MoveRight, None),
        ]);

        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Editor Element".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| EditorView::new(cx));
                let handle = cx.focus_handle();
                window.focus(&handle);
                view
            },
        )
        .unwrap();

        cx.activate(false);
    });
}
