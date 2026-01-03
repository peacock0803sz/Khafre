//! Main application component

use dioxus::prelude::*;

use crate::components::layout::SplitView;
use crate::components::preview::PreviewPane;
use crate::components::terminal::TerminalView;
use crate::state::{use_config_loader, use_terminal_init, AppState, SphinxStatus};

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize application state
    use_context_provider(AppState::default);

    // Load configuration
    use_config_loader();

    // Initialize terminal
    use_terminal_init();

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

/// Header component
#[component]
fn Header() -> Element {
    rsx! {
        header {
            style: "display: flex; align-items: center; padding: 8px 16px; background: #252526; border-bottom: 1px solid #3c3c3c;",

            // Title
            h1 {
                style: "margin: 0; font-size: 14px; font-weight: normal; flex: 1;",
                "Khafre - Sphinx Documentation Editor"
            }

            // Menu buttons (placeholder)
            div {
                style: "display: flex; gap: 8px;",

                button {
                    style: "padding: 4px 8px; background: transparent; border: 1px solid #3c3c3c; color: #d4d4d4; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    "Open Project"
                }

                button {
                    style: "padding: 4px 8px; background: transparent; border: 1px solid #3c3c3c; color: #d4d4d4; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    "Settings"
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
