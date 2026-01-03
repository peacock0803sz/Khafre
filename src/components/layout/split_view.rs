//! Split view component with draggable divider

use dioxus::prelude::*;

/// Split view with draggable divider
#[component]
pub fn SplitView(left: Element, right: Element) -> Element {
    let mut split_ratio = use_signal(|| 50.0_f64);
    let mut is_dragging = use_signal(|| false);

    let handle_mouse_down = move |_| {
        is_dragging.set(true);
    };

    let handle_mouse_up = move |_| {
        is_dragging.set(false);
    };

    let handle_mouse_move = move |e: MouseEvent| {
        if *is_dragging.read() {
            let x = e.client_coordinates().x;
            // TODO: Get container width and calculate ratio
            // For now, use a simple approximation
            let new_ratio = (x / 12.0).clamp(20.0, 80.0);
            split_ratio.set(new_ratio);
        }
    };

    rsx! {
        div {
            style: "display: flex; width: 100%; height: 100%;",
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            onmouseleave: handle_mouse_up,

            // Left pane
            div {
                style: "width: {split_ratio}%; height: 100%; overflow: hidden;",
                {left}
            }

            // Divider
            div {
                style: "width: 4px; background: #3c3c3c; cursor: col-resize; flex-shrink: 0;",
                onmousedown: handle_mouse_down,

                // Visual indicator
                div {
                    style: "width: 2px; height: 40px; background: #606060; margin: auto; margin-top: calc(50% - 20px); border-radius: 1px;",
                }
            }

            // Right pane
            div {
                style: "flex: 1; height: 100%; overflow: hidden;",
                {right}
            }
        }
    }
}
