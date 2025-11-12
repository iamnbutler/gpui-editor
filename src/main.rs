#![allow(unused, dead_code)]

use gpui::*;
use std::ops::Range as StdRange;

mod editor;
mod gap_buffer;
mod text_buffer;
use editor::{CursorPosition, Editor, EditorConfig};
use gap_buffer::GapBuffer;
use text_buffer::TextBuffer;

actions!(
    editor_view,
    [
        MoveUp,
        MoveDown,
        MoveLeft,
        MoveRight,
        Backspace,
        Delete,
        InsertNewline
    ]
);

struct EditorView {
    focus_handle: FocusHandle,
    buffer: GapBuffer,
    cursor_position: CursorPosition,
}

impl EditorView {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let initial_text = r#"use std::collections::HashMap;

fn main() {
    println!("Hello, world!");

    // Create some variables
    let x = 42;
    let name = "Alice";
    let mut count = 0;

    // Loop example
    for i in 0..10 {
        println!("Count: {}", i);
        count += 1;
    }

    // HashMap example
    let mut scores = HashMap::new();
    scores.insert("Blue", 10);
    scores.insert("Yellow", 50);

    println!("Final count: {}", count);
}"#;

        Self {
            focus_handle,
            buffer: GapBuffer::from_text(initial_text),
            cursor_position: CursorPosition { row: 0, col: 0 },
        }
    }

    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            // Clamp column to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self.cursor_position.col.min(line_len);
            cx.notify();
        }
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            self.cursor_position.row += 1;
            // Clamp column to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
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
            self.cursor_position.col = self.buffer.line_len(self.cursor_position.row);
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        let current_line_len = self.buffer.line_len(self.cursor_position.row);

        if self.cursor_position.col < current_line_len {
            self.cursor_position.col += 1;
        } else if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Move to start of next line
            self.cursor_position.row += 1;
            self.cursor_position.col = 0;
        }
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        // Move gap to cursor position
        let pos = self
            .buffer
            .cursor_to_position(self.cursor_position.row, self.cursor_position.col);
        self.buffer.move_gap_to(pos);
        self.buffer.delete_backward();

        // Update cursor position
        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
        } else if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            self.cursor_position.col = self.buffer.line_len(self.cursor_position.row);
        }

        cx.notify();
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        // Move gap to cursor position
        let pos = self
            .buffer
            .cursor_to_position(self.cursor_position.row, self.cursor_position.col);
        self.buffer.move_gap_to(pos);
        self.buffer.delete_forward();

        cx.notify();
    }

    fn insert_newline(&mut self, _: &InsertNewline, _: &mut Window, cx: &mut Context<Self>) {
        self.insert_text("\n", cx);
        self.cursor_position.row += 1;
        self.cursor_position.col = 0;
        cx.notify();
    }

    fn insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        // Move gap to cursor position
        let pos = self
            .buffer
            .cursor_to_position(self.cursor_position.row, self.cursor_position.col);
        self.buffer.move_gap_to(pos);

        // Insert text
        for ch in text.chars() {
            self.buffer.insert_char(ch);
            if ch == '\n' {
                self.cursor_position.row += 1;
                self.cursor_position.col = 0;
            } else {
                self.cursor_position.col += 1;
            }
        }
    }
}

impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range: StdRange<usize>,
        _adjusted_range: &mut Option<StdRange<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.buffer.to_string();
        let start = range.start.min(text.len());
        let end = range.end.min(text.len());
        Some(text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let pos = self
            .buffer
            .cursor_to_position(self.cursor_position.row, self.cursor_position.col);
        Some(UTF16Selection {
            range: pos..pos,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<StdRange<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // Not implementing marked text for now
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<StdRange<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pos = self
            .buffer
            .cursor_to_position(self.cursor_position.row, self.cursor_position.col);

        let range = range.unwrap_or(pos..pos);

        // Delete the range first if it's not empty
        if range.start < range.end {
            self.buffer.delete_range(range.start, range.end);
        }

        // Insert the new text
        self.buffer.insert(range.start, text);

        // Update cursor position
        let new_pos = range.start + text.len();
        let (row, col) = self.buffer.position_to_cursor(new_pos);
        self.cursor_position.row = row;
        self.cursor_position.col = col;

        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<StdRange<usize>>,
        new_text: &str,
        new_selected_range: Option<StdRange<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range(range, new_text, window, cx);

        // Update cursor if new selection is provided
        if let Some(selection) = new_selected_range {
            let (row, col) = self.buffer.position_to_cursor(selection.start);
            self.cursor_position.row = row;
            self.cursor_position.col = col;
        }
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: StdRange<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        None // Not implementing IME positioning for now
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None // Not implementing point-to-character mapping for now
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

        let editor_element = Editor::new("editor", self.buffer.all_lines())
            .config(config)
            .cursor_position(self.cursor_position);

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::move_left))
            .on_action(cx.listener(Self::move_right))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::insert_newline))
            .child(EditorElement {
                entity: cx.entity().clone(),
                editor_element,
            })
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
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("enter", InsertNewline, None),
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

struct EditorElement {
    entity: Entity<EditorView>,
    editor_element: Editor,
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = <Editor as Element>::RequestLayoutState;
    type PrepaintState = <Editor as Element>::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.editor_element.id()
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        self.editor_element.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.editor_element
            .request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout_state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.editor_element
            .prepaint(id, inspector_id, bounds, request_layout_state, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout_state: &mut Self::RequestLayoutState,
        prepaint_state: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Paint the editor element first
        self.editor_element.paint(
            id,
            inspector_id,
            bounds,
            request_layout_state,
            prepaint_state,
            window,
            cx,
        );

        // Then handle input if focused
        self.entity.read_with(cx, |view, _| {
            if view.focus_handle.is_focused(window) {
                let input_handler = ElementInputHandler::new(bounds, self.entity.clone());
                window.handle_input(&view.focus_handle, input_handler, cx);
            }
        });
    }
}
