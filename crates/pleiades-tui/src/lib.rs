//! Terminal user interface for Pleiades.
//!
//! Built on ratatui and crossterm for a beautiful terminal experience
//! with markdown rendering, syntax highlighting, and responsive layout.

pub mod app;
pub mod render;
pub mod theme;

pub use app::TuiApp;
pub use theme::Theme;
