//! Terminal view component
//!
//! This component will render the terminal using alacritty_terminal backend.
//! For now, it's a placeholder that will be implemented in Phase 2-3.

use dioxus::prelude::*;

/// Terminal view component
#[component]
pub fn TerminalView() -> Element {
    rsx! {
        div {
            style: "width: 100%; height: 100%; background: #1e1e1e; padding: 8px; font-family: 'Menlo', 'Monaco', 'Courier New', monospace; font-size: 14px;",

            // Placeholder content - will be replaced with actual terminal in Phase 2-3
            div {
                style: "color: #4ec9b0;",
                "$ "
                span {
                    style: "color: #d4d4d4;",
                    "Terminal will be implemented in Phase 2-3"
                }
            }
            div {
                style: "color: #6a9955; margin-top: 8px;",
                "# Using alacritty_terminal for VT parsing"
            }
            div {
                style: "color: #6a9955;",
                "# Using portable-pty for PTY management"
            }

            // Blinking cursor placeholder
            div {
                style: "margin-top: 16px;",
                span {
                    style: "color: #4ec9b0;",
                    "$ "
                }
                span {
                    style: "background: #d4d4d4; width: 8px; height: 16px; display: inline-block; animation: blink 1s infinite;",
                }
            }
        }
    }
}
