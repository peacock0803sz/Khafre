//! Main application component

use dioxus::prelude::*;

use crate::components::layout::SplitView;
use crate::components::preview::PreviewPane;
use crate::components::terminal::TerminalView;
use crate::state::AppState;

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize application state
    use_context_provider(AppState::default);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e; color: #d4d4d4;",

            // Header
            header {
                style: "padding: 8px 16px; background: #252526; border-bottom: 1px solid #3c3c3c;",
                h1 {
                    style: "margin: 0; font-size: 14px; font-weight: normal;",
                    "Khafre - Sphinx Documentation Editor"
                }
            }

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
            footer {
                style: "padding: 4px 16px; background: #007acc; font-size: 12px; color: white;",
                "Ready"
            }
        }
    }
}
