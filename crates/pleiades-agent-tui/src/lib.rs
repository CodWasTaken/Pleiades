//! Terminal user interface for Pleiades.
//!
//! Built on ratatui and crossterm for a beautiful terminal experience
//! with markdown rendering, syntax highlighting, and responsive layout.

pub mod app;
pub mod input;
pub mod markdown;
pub mod render;
pub mod state;
pub mod terminal;
pub mod theme;
pub mod ui;

pub use app::TuiApp;
pub use render::TerminalRenderer;
pub use theme::Theme;
