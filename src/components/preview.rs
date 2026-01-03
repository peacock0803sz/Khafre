//! Preview pane component
//!
//! This component will display the Sphinx documentation preview.
//! For now, it's a placeholder that will be implemented in Phase 4.

use dioxus::prelude::*;

/// Preview pane component
#[component]
pub fn PreviewPane() -> Element {
    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #ffffff; display: flex; flex-direction: column; align-items: center; justify-content: center;",

            // Placeholder content
            div {
                style: "text-align: center; color: #666;",

                h2 {
                    style: "margin: 0 0 16px 0; font-size: 24px; font-weight: normal;",
                    "Sphinx Preview"
                }

                p {
                    style: "margin: 0 0 8px 0;",
                    "Preview will be implemented in Phase 4"
                }

                p {
                    style: "margin: 0; font-size: 12px; color: #999;",
                    "Using WebView (wry) or external browser"
                }
            }
        }
    }
}
