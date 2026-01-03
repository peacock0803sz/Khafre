//! Preview pane component
//!
//! This component displays the Sphinx documentation preview.
//! It shows an iframe when the Sphinx server is running,
//! or a placeholder with controls when not running.

use dioxus::prelude::*;

use crate::state::{AppState, SphinxStatus};

/// Preview pane component
#[component]
pub fn PreviewPane() -> Element {
    let app_state = use_context::<AppState>();
    let sphinx_state = app_state.sphinx.read();

    match sphinx_state.status {
        SphinxStatus::Running => {
            if let Some(port) = sphinx_state.port {
                // Show iframe with Sphinx preview
                rsx! {
                    PreviewFrame { port }
                }
            } else {
                rsx! {
                    PreviewPlaceholder {
                        message: "Sphinx server running but port unknown",
                        status: "error",
                    }
                }
            }
        }
        SphinxStatus::Starting => {
            rsx! {
                PreviewPlaceholder {
                    message: "Starting Sphinx server...",
                    status: "loading",
                }
            }
        }
        SphinxStatus::Building => {
            if let Some(port) = sphinx_state.port {
                // Still show iframe but with building indicator
                rsx! {
                    div {
                        style: "width: 100%; height: 100%; position: relative;",

                        PreviewFrame { port }

                        // Building overlay
                        div {
                            style: "position: absolute; top: 8px; right: 8px; background: #ffc107; color: #000; padding: 4px 8px; border-radius: 4px; font-size: 12px;",
                            "Building..."
                        }
                    }
                }
            } else {
                rsx! {
                    PreviewPlaceholder {
                        message: "Building documentation...",
                        status: "loading",
                    }
                }
            }
        }
        SphinxStatus::Error(ref msg) => {
            rsx! {
                PreviewPlaceholder {
                    message: "Error: {msg}",
                    status: "error",
                }
            }
        }
        SphinxStatus::Stopped => {
            rsx! {
                PreviewPlaceholder {
                    message: "Sphinx server not running",
                    status: "stopped",
                }
            }
        }
    }
}

/// Preview iframe component
#[component]
fn PreviewFrame(port: u16) -> Element {
    let url = format!("http://127.0.0.1:{}", port);

    rsx! {
        div {
            style: "width: 100%; height: 100%; display: flex; flex-direction: column;",

            // Toolbar
            div {
                style: "display: flex; align-items: center; padding: 4px 8px; background: #f5f5f5; border-bottom: 1px solid #ddd; gap: 8px;",

                // URL display
                div {
                    style: "flex: 1; font-size: 12px; color: #666; font-family: monospace;",
                    "{url}"
                }

                // Refresh button
                button {
                    style: "padding: 4px 8px; border: 1px solid #ccc; border-radius: 4px; background: #fff; cursor: pointer; font-size: 12px;",
                    onclick: move |_| {
                        // Refresh iframe
                        // TODO: Implement iframe refresh
                    },
                    "Refresh"
                }

                // Open in browser button
                button {
                    style: "padding: 4px 8px; border: 1px solid #ccc; border-radius: 4px; background: #fff; cursor: pointer; font-size: 12px;",
                    onclick: move |_| {
                        let url = format!("http://127.0.0.1:{}", port);
                        let _ = open::that(&url);
                    },
                    "Open in Browser"
                }
            }

            // iframe
            iframe {
                style: "flex: 1; width: 100%; border: none;",
                src: "{url}",
                // Sandbox for security, but allow scripts and same-origin
                sandbox: "allow-scripts allow-same-origin",
            }
        }
    }
}

/// Placeholder component for when preview is not available
#[component]
fn PreviewPlaceholder(message: String, status: String) -> Element {
    let (icon, color) = match status.as_str() {
        "loading" => ("⏳", "#666"),
        "error" => ("❌", "#dc3545"),
        "stopped" => ("📄", "#666"),
        _ => ("ℹ️", "#666"),
    };

    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #f8f9fa; display: flex; flex-direction: column; align-items: center; justify-content: center;",

            div {
                style: "text-align: center; max-width: 400px; padding: 32px;",

                // Icon
                div {
                    style: "font-size: 48px; margin-bottom: 16px;",
                    "{icon}"
                }

                // Message
                p {
                    style: "margin: 0 0 24px 0; color: {color}; font-size: 16px;",
                    "{message}"
                }

                // Instructions when stopped
                if status == "stopped" {
                    div {
                        style: "color: #999; font-size: 14px; line-height: 1.6;",
                        p {
                            style: "margin: 0 0 8px 0;",
                            "To start the preview:"
                        }
                        ol {
                            style: "margin: 0; padding-left: 20px; text-align: left;",
                            li { "Open a Sphinx project" }
                            li { "Configure Python interpreter" }
                            li { "Start sphinx-autobuild" }
                        }
                    }
                }
            }
        }
    }
}
