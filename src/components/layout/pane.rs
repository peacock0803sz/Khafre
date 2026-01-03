//! Pane container component

use dioxus::prelude::*;

/// A simple pane container
#[component]
pub fn Pane(children: Element) -> Element {
    rsx! {
        div {
            style: "width: 100%; height: 100%; overflow: hidden;",
            {children}
        }
    }
}
