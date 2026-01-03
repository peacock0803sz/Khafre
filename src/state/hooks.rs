//! Custom hooks for state management

use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::Mutex;

use crate::services::config::load_config;
use crate::services::sphinx::{SphinxEvent, SphinxManager};
use crate::services::terminal::TerminalManager;
use crate::types::config::Config;

use super::{AppState, SphinxState, SphinxStatus, TerminalState};

/// Initialize terminal hook
///
/// Creates and manages the terminal manager lifecycle.
/// Re-creates terminal when project path changes.
pub fn use_terminal_init() {
    let app_state = use_context::<AppState>();

    // Track project path changes to recreate terminal
    let project_path = app_state.project_path.read().clone();

    use_effect(move || {
        let mut app_state = app_state.clone();
        let project_path = project_path.clone();

        spawn(async move {
            // Get terminal config
            let config = app_state.config.read();
            let shell = config.as_ref().and_then(|c| c.terminal.shell.clone());
            drop(config);

            // Create terminal manager with project directory as cwd
            match TerminalManager::new(80, 24, shell.as_deref(), project_path.as_deref()) {
                Ok(manager) => {
                    let manager = Arc::new(Mutex::new(manager));
                    app_state.terminal_manager.set(Some(manager));

                    // Update terminal state
                    app_state.terminal.set(TerminalState {
                        session_id: Some(uuid::Uuid::new_v4().to_string()),
                        ready: true,
                        cols: 80,
                        rows: 24,
                    });

                    log::info!("Terminal initialized with cwd: {:?}", project_path);
                }
                Err(e) => {
                    log::error!("Failed to initialize terminal: {}", e);
                }
            }
        });
    });
}

/// Load configuration hook
///
/// Loads configuration from disk and updates state.
pub fn use_config_loader() {
    let mut app_state = use_context::<AppState>();

    use_effect(move || {
        spawn(async move {
            match load_config() {
                Ok(config) => {
                    app_state.config.set(Some(config));
                    log::info!("Configuration loaded");
                }
                Err(e) => {
                    log::warn!("Failed to load config, using defaults: {}", e);
                    app_state.config.set(Some(Config::default()));
                }
            }
        });
    });
}

/// Sphinx manager hook result
pub struct UseSphinx {
    /// Get current port
    pub port: Option<u16>,

    /// Get current status
    pub status: SphinxStatus,
}

/// Use Sphinx server hook
pub fn use_sphinx() -> UseSphinx {
    let app_state = use_context::<AppState>();
    let sphinx_state = app_state.sphinx.read();

    UseSphinx {
        port: sphinx_state.port,
        status: sphinx_state.status.clone(),
    }
}

/// Start Sphinx server
pub fn start_sphinx(app_state: AppState, project_path: String, session_id: String) {
    let mut app_state = app_state;

    spawn(async move {
        // Update status to starting
        app_state.sphinx.set(SphinxState {
            port: None,
            status: SphinxStatus::Starting,
            last_build: None,
        });

        // Get config
        let config = app_state.config.read();
        let config = config.as_ref().cloned().unwrap_or_default();
        drop(config);

        let config = app_state.config.read().as_ref().cloned().unwrap_or_default();

        // Create Sphinx manager
        let (mut manager, mut event_rx) = SphinxManager::new();

        // Start server
        match manager.start(
            session_id.clone(),
            &project_path,
            &config.sphinx.source_dir,
            &config.sphinx.build_dir,
            &config.python.interpreter,
            config.sphinx.server.port,
            config.sphinx.extra_args.clone(),
        ) {
            Ok(port) => {
                log::info!("Sphinx server starting on port {}", port);

                // Handle events in background
                let mut app_state_events = app_state.clone();
                spawn(async move {
                    while let Some(event) = event_rx.recv().await {
                        match event {
                            SphinxEvent::Started { port, .. } => {
                                app_state_events.sphinx.set(SphinxState {
                                    port: Some(port),
                                    status: SphinxStatus::Running,
                                    last_build: None,
                                });
                            }
                            SphinxEvent::Built { .. } => {
                                let state = app_state_events.sphinx.read().clone();
                                app_state_events.sphinx.set(SphinxState {
                                    status: SphinxStatus::Running,
                                    last_build: Some(chrono_now()),
                                    ..state
                                });
                            }
                            SphinxEvent::Error { message, .. } => {
                                let state = app_state_events.sphinx.read().clone();
                                app_state_events.sphinx.set(SphinxState {
                                    status: SphinxStatus::Error(message),
                                    ..state
                                });
                            }
                            SphinxEvent::Stopped { .. } => {
                                app_state_events.sphinx.set(SphinxState::default());
                            }
                        }
                    }
                });
            }
            Err(e) => {
                log::error!("Failed to start Sphinx server: {}", e);
                app_state.sphinx.set(SphinxState {
                    port: None,
                    status: SphinxStatus::Error(e.to_string()),
                    last_build: None,
                });
            }
        }
    });
}

/// Stop Sphinx server
pub fn stop_sphinx(mut app_state: AppState) {
    app_state.sphinx.set(SphinxState::default());
    log::info!("Sphinx server stopped");
}

/// Get current timestamp string
fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Terminal resize hook
pub fn use_terminal_resize() -> impl Fn(u16, u16) {
    let app_state = use_context::<AppState>();

    move |cols: u16, rows: u16| {
        let mut app_state = app_state.clone();

        spawn(async move {
            if let Some(ref manager_arc) = *app_state.terminal_manager.read() {
                let mut manager = manager_arc.lock().await;
                if let Err(e) = manager.resize(cols, rows) {
                    log::error!("Failed to resize terminal: {}", e);
                } else {
                    // Update terminal state
                    let state = app_state.terminal.read().clone();
                    app_state.terminal.set(TerminalState {
                        cols,
                        rows,
                        ..state
                    });
                }
            }
        });
    }
}
