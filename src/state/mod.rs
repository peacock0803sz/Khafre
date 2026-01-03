//! Application state management

use dioxus::prelude::*;

use crate::types::config::Config;

/// Main application state
#[derive(Clone, Default)]
pub struct AppState {
    /// Application configuration
    pub config: Signal<Option<Config>>,

    /// Sphinx server state
    pub sphinx: Signal<SphinxState>,

    /// Terminal state
    pub terminal: Signal<TerminalState>,
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
}
