//! Application state management

pub mod hooks;

use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::Mutex;

use crate::services::terminal::TerminalManager;
use crate::types::color_scheme::ColorScheme;
use crate::types::config::Config;

pub use hooks::{use_config_loader, use_terminal_init, use_terminal_resize, start_sphinx, stop_sphinx};

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Signal<Option<Config>>,

    /// Current project path
    pub project_path: Signal<Option<String>>,

    /// Sphinx server state
    pub sphinx: Signal<SphinxState>,

    /// Terminal state
    pub terminal: Signal<TerminalState>,

    /// Terminal manager (wrapped in Arc<Mutex> for thread safety)
    pub terminal_manager: Signal<Option<Arc<Mutex<TerminalManager>>>>,

    /// Current color scheme
    pub color_scheme: Signal<ColorScheme>,

    /// System theme is dark
    pub is_dark_theme: Signal<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        // Detect system theme at startup
        let is_dark = crate::services::theme::detect_system_theme();
        let color_scheme = if is_dark {
            ColorScheme::dark()
        } else {
            ColorScheme::light()
        };

        Self {
            config: Signal::new(None),
            project_path: Signal::new(None),
            sphinx: Signal::new(SphinxState::default()),
            terminal: Signal::new(TerminalState::default()),
            terminal_manager: Signal::new(None),
            color_scheme: Signal::new(color_scheme),
            is_dark_theme: Signal::new(is_dark),
        }
    }
}

/// Sphinx server state
#[derive(Clone, Default)]
pub struct SphinxState {
    /// Server port (None if not running)
    pub port: Option<u16>,

    /// Current status
    pub status: SphinxStatus,

    /// Last build timestamp
    pub last_build: Option<String>,
}

/// Sphinx server status
#[derive(Clone, Default, PartialEq)]
pub enum SphinxStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Building,
    Error(String),
}

/// Terminal state
#[derive(Clone, Default)]
pub struct TerminalState {
    /// Session ID
    pub session_id: Option<String>,

    /// Whether terminal is ready
    pub ready: bool,

    /// Terminal dimensions
    pub cols: u16,
    pub rows: u16,
}

impl TerminalState {
    /// Create a new terminal state with default dimensions
    pub fn new() -> Self {
        Self {
            session_id: None,
            ready: false,
            cols: 80,
            rows: 24,
        }
    }
}
