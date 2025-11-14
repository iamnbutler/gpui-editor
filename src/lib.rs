//! A standalone editor component for GPUI
//!
//! This crate provides a text editor widget for GPUI applications with syntax highlighting support.
//!
//! # Architecture
//!
//! The editor is structured in three layers:
//!
//! - **Editor**: The core data model and editing operations
//! - **EditorElement**: The GPUI element that renders an Editor
//! - **EditorView**: A complete view with keyboard handling (see examples)

pub mod editor;
pub mod element;
pub mod syntax_highlighter;
pub mod text_buffer;

// Internal modules
mod gap_buffer;
mod meta_line;

// Re-export main types
pub use editor::{CursorPosition, Editor, EditorConfig};
pub use element::EditorElement;
pub use meta_line::{Language, MetaLine, Selection};
pub use syntax_highlighter::SyntaxHighlighter;
pub use text_buffer::{SimpleBuffer, TextBuffer};

// Re-export gpui for convenience
pub use gpui;
