//! Main application component

use dioxus::prelude::*;

use crate::components::layout::SplitView;
use crate::components::preview::PreviewPane;
use crate::components::terminal::TerminalView;
use crate::state::{
    start_sphinx, stop_sphinx, use_config_loader, use_terminal_init, AppState, SphinxStatus,
};

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize application state
    use_context_provider(AppState::default);

    // Load configuration
    use_config_loader();

    // Initialize terminal
    use_terminal_init();

    // Auto-start Sphinx when project is selected and config is loaded
    use_sphinx_auto_start();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e; color: #d4d4d4;",

            // Header
            Header {}

            // Main content area with split view
            main {
                style: "flex: 1; display: flex; overflow: hidden;",

                SplitView {
                    left: rsx! {
                        TerminalView {}
                    },
                    right: rsx! {
                        PreviewPane {}
                    },
                }
            }

            // Status bar
            StatusBar {}
        }
    }
}

/// Hook to auto-start Sphinx when conditions are met
fn use_sphinx_auto_start() {
    let app_state = use_context::<AppState>();

    use_effect(move || {
        let config = app_state.config.read();
        let project_path = app_state.project_path.read();
        let sphinx_status = app_state.sphinx.read().status.clone();

        // Auto-start if we have config, project path, and Sphinx is stopped
        if config.is_some() && project_path.is_some() && sphinx_status == SphinxStatus::Stopped {
            let project_path = project_path.clone().unwrap();
            let session_id = uuid::Uuid::new_v4().to_string();
            start_sphinx(app_state.clone(), project_path, session_id);
        }
    });
}

/// Header component
#[component]
fn Header() -> Element {
    let app_state = use_context::<AppState>();
    let project_path = app_state.project_path.read().clone();
    let sphinx_state = app_state.sphinx.read().clone();
    let config_loaded = app_state.config.read().is_some();

    let sphinx_running = matches!(
        sphinx_state.status,
        SphinxStatus::Running | SphinxStatus::Starting | SphinxStatus::Building
    );

    // Project selection handler
    let handle_open_project = {
        let app_state = app_state.clone();
        move |_| {
            let mut app_state = app_state.clone();
            spawn(async move {
                if let Some(path) = rfd::AsyncFileDialog::new()
                    .set_title("Select Sphinx Project Folder")
                    .pick_folder()
                    .await
                {
                    let path_str = path.path().to_string_lossy().to_string();
                    app_state.project_path.set(Some(path_str));
                }
            });
        }
    };

    // Start Sphinx handler
    let handle_start_sphinx = {
        let app_state = app_state.clone();
        move |_| {
            let project_path = app_state.project_path.read().clone();
            if let Some(path) = project_path {
                let session_id = uuid::Uuid::new_v4().to_string();
                start_sphinx(app_state.clone(), path, session_id);
            }
        }
    };

    // Stop Sphinx handler
    let handle_stop_sphinx = {
        let app_state = app_state.clone();
        move |_| {
            stop_sphinx(app_state.clone());
        }
    };

    // Open in browser handler
    let handle_open_browser = {
        let port = sphinx_state.port;
        move |_| {
            if let Some(port) = port {
                let url = format!("http://127.0.0.1:{}", port);
                let _ = open::that(&url);
            }
        }
    };

    rsx! {
        header {
            style: "display: flex; align-items: center; padding: 8px 16px; background: #252526; border-bottom: 1px solid #3c3c3c; gap: 16px;",

            // Title
            span {
                style: "font-size: 14px; font-weight: 600;",
                "Khafre"
            }

            // Project path
            if let Some(ref path) = project_path {
                span {
                    style: "font-size: 12px; color: #888; max-width: 400px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{path}"
                }
            }

            // Spacer
            span {
                style: "flex: 1;",
            }

            // Status indicators
            if !config_loaded {
                span {
                    style: "font-size: 11px; color: #ffc107;",
                    "Loading config..."
                }
            }

            match &sphinx_state.status {
                SphinxStatus::Starting => rsx! {
                    span {
                        style: "font-size: 11px; color: #ffc107;",
                        "Starting..."
                    }
                },
                SphinxStatus::Building => rsx! {
                    span {
                        style: "font-size: 11px; color: #ffc107;",
                        "Building..."
                    }
                },
                SphinxStatus::Running => rsx! {
                    span {
                        style: "font-size: 11px; color: #4caf50;",
                        "Preview Ready"
                    }
                },
                SphinxStatus::Error(msg) => rsx! {
                    span {
                        style: "font-size: 11px; color: #f44336; max-width: 200px; overflow: hidden; text-overflow: ellipsis;",
                        title: "{msg}",
                        "Error"
                    }
                },
                SphinxStatus::Stopped => rsx! {},
            }

            // Control buttons
            div {
                style: "display: flex; gap: 8px;",

                // Open Project button
                button {
                    style: "padding: 4px 12px; background: #0e639c; border: none; color: white; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    onclick: handle_open_project,
                    "Open Project"
                }

                // Start/Stop Sphinx button
                if project_path.is_some() && config_loaded {
                    if sphinx_running {
                        button {
                            style: "padding: 4px 12px; background: #d32f2f; border: none; color: white; border-radius: 4px; cursor: pointer; font-size: 12px;",
                            onclick: handle_stop_sphinx,
                            "Stop Preview"
                        }
                    } else {
                        button {
                            style: "padding: 4px 12px; background: #388e3c; border: none; color: white; border-radius: 4px; cursor: pointer; font-size: 12px;",
                            onclick: handle_start_sphinx,
                            "Start Preview"
                        }
                    }
                }

                // Open in Browser button
                if sphinx_state.port.is_some() {
                    button {
                        style: "padding: 4px 12px; background: transparent; border: 1px solid #3c3c3c; color: #d4d4d4; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: handle_open_browser,
                        "Open in Browser"
                    }
                }
            }
        }
    }
}

/// Status bar component
#[component]
fn StatusBar() -> Element {
    let app_state = use_context::<AppState>();
    let terminal_state = app_state.terminal.read();
    let sphinx_state = app_state.sphinx.read();

    let terminal_status = if terminal_state.ready {
        format!("Terminal: {}x{}", terminal_state.cols, terminal_state.rows)
    } else {
        "Terminal: Initializing...".to_string()
    };

    let sphinx_status = match &sphinx_state.status {
        SphinxStatus::Stopped => "Sphinx: Stopped".to_string(),
        SphinxStatus::Starting => "Sphinx: Starting...".to_string(),
        SphinxStatus::Running => {
            if let Some(port) = sphinx_state.port {
                format!("Sphinx: Running (port {})", port)
            } else {
                "Sphinx: Running".to_string()
            }
        }
        SphinxStatus::Building => "Sphinx: Building...".to_string(),
        SphinxStatus::Error(msg) => format!("Sphinx: Error - {}", msg),
    };

    rsx! {
        footer {
            style: "display: flex; padding: 4px 16px; background: #007acc; font-size: 12px; color: white; gap: 16px;",

            // Terminal status
            span {
                "{terminal_status}"
            }

            // Separator
            span {
                style: "opacity: 0.5;",
                "|"
            }

            // Sphinx status
            span {
                "{sphinx_status}"
            }

            // Spacer
            span {
                style: "flex: 1;",
            }

            // Build timestamp
            if let Some(ref timestamp) = sphinx_state.last_build {
                span {
                    style: "opacity: 0.7;",
                    "Last build: {timestamp}"
                }
            }
        }
    }
}
