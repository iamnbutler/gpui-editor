#![allow(unused, dead_code)]

use gpui::*;
use std::ops::Range as StdRange;

mod editor;
mod gap_buffer;
mod syntax_highlighter;
mod text_buffer;
use editor::{CursorPosition, Editor, EditorConfig};
use gap_buffer::GapBuffer;
use text_buffer::{SimpleBuffer, TextBuffer};

actions!(
    editor_view,
    [
        MoveUp,
        MoveDown,
        MoveLeft,
        MoveRight,
        Backspace,
        Delete,
        InsertNewline,
        NextTheme,
        PreviousTheme,
        NextLanguage,
        PreviousLanguage
    ]
);

struct EditorView {
    focus_handle: FocusHandle,
    buffer: GapBuffer,
    cursor_position: CursorPosition,
    editor: editor::Editor,
    current_theme_index: usize,
    available_themes: Vec<String>,
    current_language_index: usize,
    available_languages: Vec<(String, String, String)>, // (name, extension, sample_code)
}

impl EditorView {
    fn get_sample_languages() -> Vec<(String, String, String)> {
        vec![
            (
                "Rust".to_string(),
                "rs".to_string(),
                r#"use std::collections::HashMap;

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
}"#.to_string(),
            ),
            (
                "JavaScript".to_string(),
                "js".to_string(),
                r#"// JavaScript sample code
const express = require('express');
const app = express();
const port = 3000;

// Define a route
app.get('/', (req, res) => {
    res.send('Hello World!');
});

// Async function example
async function fetchData(url) {
    try {
        const response = await fetch(url);
        const data = await response.json();
        console.log('Data:', data);
        return data;
    } catch (error) {
        console.error('Error fetching data:', error);
    }
}

// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});"#.to_string(),
            ),
            (
                "Python".to_string(),
                "py".to_string(),
                r#"#!/usr/bin/env python3
import asyncio
import json
from typing import List, Dict, Optional

class DataProcessor:
    """A sample data processing class."""

    def __init__(self, name: str):
        self.name = name
        self.data: List[Dict] = []

    def process(self, item: Dict) -> Optional[Dict]:
        """Process a single data item."""
        if not item:
            return None

        # Transform the data
        result = {
            'id': item.get('id', 0),
            'processed_by': self.name,
            'timestamp': item.get('timestamp'),
            'value': item.get('value', 0) * 2
        }

        return result

async def main():
    """Main async function."""
    processor = DataProcessor("Sample Processor")

    # Sample data
    items = [
        {'id': 1, 'value': 10},
        {'id': 2, 'value': 20},
        {'id': 3, 'value': 30}
    ]

    for item in items:
        result = processor.process(item)
        print(f"Processed: {result}")

    await asyncio.sleep(1)
    print("Processing complete!")

if __name__ == "__main__":
    asyncio.run(main())"#.to_string(),
            ),
            (
                "HTML".to_string(),
                "html".to_string(),
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sample HTML Page</title>
    <style>
        body {
            font-family: 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
        }
        h1 {
            color: #333;
            border-bottom: 2px solid #667eea;
            padding-bottom: 10px;
        }
        button {
            background: #667eea;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
        }
        button:hover {
            background: #5a67d8;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome to the Sample Page</h1>
        <p>This is a sample HTML page with embedded CSS and JavaScript.</p>
        <button onclick="showMessage()">Click Me!</button>
        <div id="message"></div>
    </div>

    <script>
        function showMessage() {
            const messageDiv = document.getElementById('message');
            messageDiv.innerHTML = '<p style="color: green;">Button clicked at ' + new Date().toLocaleTimeString() + '</p>';
        }
    </script>
</body>
</html>"#.to_string(),
            ),
            (
                "Go".to_string(),
                "go".to_string(),
                r#"package main

import (
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "time"
)

// User represents a user in the system
type User struct {
    ID        int       `json:"id"`
    Name      string    `json:"name"`
    Email     string    `json:"email"`
    CreatedAt time.Time `json:"created_at"`
}

// UserService handles user-related operations
type UserService struct {
    users map[int]*User
}

// NewUserService creates a new UserService instance
func NewUserService() *UserService {
    return &UserService{
        users: make(map[int]*User),
    }
}

// GetUser retrieves a user by ID
func (s *UserService) GetUser(id int) (*User, error) {
    user, exists := s.users[id]
    if !exists {
        return nil, fmt.Errorf("user with ID %d not found", id)
    }
    return user, nil
}

func main() {
    service := NewUserService()

    // Sample user
    user := &User{
        ID:        1,
        Name:      "John Doe",
        Email:     "john@example.com",
        CreatedAt: time.Now(),
    }

    service.users[user.ID] = user

    // Start HTTP server
    http.HandleFunc("/user", func(w http.ResponseWriter, r *http.Request) {
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(user)
    })

    fmt.Println("Server starting on :8080...")
    log.Fatal(http.ListenAndServe(":8080", nil))
}"#.to_string(),
            ),
        ]
    }

    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let available_languages = Self::get_sample_languages();
        let current_language_index = 0;

        let (_, _, initial_text) = &available_languages[current_language_index];

        let buffer = GapBuffer::from_text(&initial_text);
        let lines = buffer.all_lines();

        let mut editor = editor::Editor::new("editor", lines);
        editor.set_language("Rust".to_string());

        let available_themes = vec![
            "Monokai".to_string(),
            "base16-ocean.dark".to_string(),
            "base16-ocean.light".to_string(),
            "InspiredGitHub".to_string(),
            "Solarized (dark)".to_string(),
            "Solarized (light)".to_string(),
        ];

        editor.set_theme("Monokai");

        Self {
            focus_handle,
            buffer,
            cursor_position: CursorPosition { row: 0, col: 0 },
            editor,
            current_theme_index: 0,
            available_themes,
            current_language_index,
            available_languages,
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

        // Update editor with new buffer content
        self.editor = editor::Editor::new("editor", self.buffer.all_lines());
        self.editor.set_language("Rust".to_string());
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        self.editor.set_cursor_position(self.cursor_position);
    }

    fn next_theme(&mut self, _: &NextTheme, _: &mut Window, cx: &mut Context<Self>) {
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        cx.notify();
    }

    fn previous_theme(&mut self, _: &PreviousTheme, _: &mut Window, cx: &mut Context<Self>) {
        if self.current_theme_index == 0 {
            self.current_theme_index = self.available_themes.len() - 1;
        } else {
            self.current_theme_index -= 1;
        }
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        cx.notify();
    }

    fn next_language(&mut self, _: &NextLanguage, _: &mut Window, cx: &mut Context<Self>) {
        self.current_language_index =
            (self.current_language_index + 1) % self.available_languages.len();
        let (name, ext, sample_code) = &self.available_languages[self.current_language_index];

        // Replace buffer with new sample code
        self.buffer = GapBuffer::from_text(sample_code);
        self.cursor_position = CursorPosition { row: 0, col: 0 };

        // Update editor with new language
        self.editor = editor::Editor::new("editor", self.buffer.all_lines());
        self.editor.set_language(name.clone());
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        self.editor.set_cursor_position(self.cursor_position);

        cx.notify();
    }

    fn previous_language(&mut self, _: &PreviousLanguage, _: &mut Window, cx: &mut Context<Self>) {
        if self.current_language_index == 0 {
            self.current_language_index = self.available_languages.len() - 1;
        } else {
            self.current_language_index -= 1;
        }
        let (name, ext, sample_code) = &self.available_languages[self.current_language_index];

        // Replace buffer with new sample code
        self.buffer = GapBuffer::from_text(sample_code);
        self.cursor_position = CursorPosition { row: 0, col: 0 };

        // Update editor with new language
        self.editor = editor::Editor::new("editor", self.buffer.all_lines());
        self.editor.set_language(name.clone());
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        self.editor.set_cursor_position(self.cursor_position);

        cx.notify();
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
        // Update editor with current buffer state
        let (language_name, _, _) = &self.available_languages[self.current_language_index];
        self.editor = editor::Editor::new("editor", self.buffer.all_lines());
        self.editor.set_language(language_name.clone());
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        self.editor.set_cursor_position(self.cursor_position);

        let current_theme = &self.available_themes[self.current_theme_index];
        let (current_language, _, _) = &self.available_languages[self.current_language_index];

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_grow()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(Self::move_up))
                    .on_action(cx.listener(Self::move_down))
                    .on_action(cx.listener(Self::move_left))
                    .on_action(cx.listener(Self::move_right))
                    .on_action(cx.listener(Self::backspace))
                    .on_action(cx.listener(Self::delete))
                    .on_action(cx.listener(Self::insert_newline))
                    .on_action(cx.listener(Self::next_theme))
                    .on_action(cx.listener(Self::previous_theme))
                    .on_action(cx.listener(Self::next_language))
                    .on_action(cx.listener(Self::previous_language))
                    .child(EditorElement {
                        entity: cx.entity().clone(),
                        editor_element: self.editor.clone(),
                    }),
            )
            .child(
                // Status bar
                div()
                    .h(px(24.0))
                    .w_full()
                    .bg(rgb(0x2b2b2b))
                    .border_t_1()
                    .border_color(rgb(0x3c3c3c))
                    .flex()
                    .items_center()
                    .px_3()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .text_sm()
                            .text_color(rgb(0xaaaaaa))
                            .child(SharedString::from(format!("Theme: {}", current_theme)))
                            .child(SharedString::from(" | "))
                            .child(SharedString::from(format!(
                                "Language: {}",
                                current_language
                            )))
                            .child(SharedString::from(" | "))
                            .child(SharedString::from(format!(
                                "Ln {}, Col {}",
                                self.cursor_position.row + 1,
                                self.cursor_position.col + 1
                            )))
                            .child(SharedString::from(" | "))
                            .child(SharedString::from(
                                "Cmd+T/Shift+T: Theme | Cmd+L/Shift+L: Language",
                            )),
                    ),
            )
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
            KeyBinding::new("cmd-t", NextTheme, None),
            KeyBinding::new("cmd-shift-t", PreviousTheme, None),
            KeyBinding::new("cmd-l", NextLanguage, None),
            KeyBinding::new("cmd-shift-l", PreviousLanguage, None),
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

        // Handle mouse events
        let entity = self.entity.clone();

        window.on_mouse_event::<MouseDownEvent>(move |mouse_down, phase, window, cx| {
            if phase != DispatchPhase::Bubble {
                return;
            }

            if bounds.contains(&mouse_down.position) {
                entity.update(cx, |view, cx| {
                    // Use the view's editor to calculate cursor position
                    let new_cursor =
                        view.editor
                            .position_to_cursor(mouse_down.position, bounds, window);

                    view.cursor_position = new_cursor;
                    view.editor.set_cursor_position(new_cursor);

                    // Focus the editor when clicked
                    window.focus(&view.focus_handle);
                    cx.notify();
                });
            }
        });

        // Handle input if focused
        self.entity.read_with(cx, |view, _| {
            if view.focus_handle.is_focused(window) {
                let input_handler = ElementInputHandler::new(bounds, self.entity.clone());
                window.handle_input(&view.focus_handle, input_handler, cx);
            }
        });
    }
}
