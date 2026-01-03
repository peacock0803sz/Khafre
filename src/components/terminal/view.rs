//! Terminal view component
//!
//! This component renders the terminal using alacritty_terminal backend.
//! Features:
//! - DOM-based cell rendering with CSS styling
//! - Mouse selection support
//! - Scroll support (mouse wheel)
//! - Resize handling

use std::time::Duration;

use dioxus::prelude::*;
use dioxus::events::WheelDelta;

use super::selection::Selection;
use crate::state::{use_terminal_resize, AppState};
use crate::types::color_scheme::ColorScheme;
use crate::types::terminal::{CellInfo, CursorInfo, CursorShape, TerminalGrid};

/// Terminal view component
#[component]
pub fn TerminalView() -> Element {
    let app_state = use_context::<AppState>();
    let mut grid = use_signal(TerminalGrid::default);
    let color_scheme = use_signal(ColorScheme::default);
    let mut selection = use_signal(Selection::default);
    let resize_terminal = use_terminal_resize();

    // Cell dimensions (monospace font metrics)
    let cell_width = 9.0_f64;
    let cell_height = 18.0_f64;
    let font_size = 14;
    let font_family = "'Menlo', 'Monaco', 'Courier New', monospace";

    // Clone the terminal manager signal for use in effect
    let terminal_manager = app_state.terminal_manager.clone();

    // Update grid from terminal manager periodically
    use_effect(move || {
        let terminal_manager = terminal_manager.clone();

        spawn(async move {
            loop {
                // Update at ~30fps
                tokio::time::sleep(Duration::from_millis(33)).await;

                if let Some(ref manager_arc) = *terminal_manager.read() {
                    let manager = manager_arc.lock().await;
                    let new_grid = manager.get_grid();
                    drop(manager);
                    grid.set(new_grid);
                }
            }
        });
    });

    let current_grid = grid.read();
    let scheme = color_scheme.read();
    let current_selection = selection.read();

    // Mouse event handlers for selection
    let handle_mouse_down = move |e: MouseEvent| {
        let (row, col) = mouse_to_cell(&e, cell_width, cell_height);
        selection.write().start_at(row, col);
    };

    let handle_mouse_move = move |e: MouseEvent| {
        if selection.read().active {
            let (row, col) = mouse_to_cell(&e, cell_width, cell_height);
            selection.write().update_to(row, col);
        }
    };

    let handle_mouse_up = {
        let grid = grid.clone();
        move |_: MouseEvent| {
            let mut sel = selection.write();
            sel.complete();

            // Copy to clipboard if there's a selection
            if sel.has_selection() {
                let text = sel.get_text(&grid.read());
                if !text.is_empty() {
                    // TODO: Copy to clipboard using system clipboard API
                    log::debug!("Selected text: {}", text);
                }
            }
        }
    };

    // Scroll handler
    let handle_wheel = {
        let app_state = app_state.clone();
        move |e: WheelEvent| {
            let delta = e.delta();
            let lines = match delta {
                WheelDelta::Pixels(_, y) => (y / cell_height) as i32,
                WheelDelta::Lines(_, y) => y as i32,
                WheelDelta::Pages(_, y) => (y * 24.0) as i32,
            };

            if let Some(ref manager_arc) = *app_state.terminal_manager.read() {
                let manager_arc = manager_arc.clone();
                spawn(async move {
                    let manager = manager_arc.lock().await;
                    manager.scroll(-lines);
                });
            }
        }
    };

    // Keyboard handler
    let handle_keydown = {
        let app_state = app_state.clone();
        move |e: Event<KeyboardData>| {
            let app_state = app_state.clone();

            // Clear selection on keypress
            selection.write().clear();

            spawn(async move {
                let key_data = e.data();
                let key = key_data.key();
                let modifiers = key_data.modifiers();

                let bytes = key_to_bytes(&key, modifiers);

                if !bytes.is_empty() {
                    if let Some(ref manager_arc) = *app_state.terminal_manager.read() {
                        let mut manager = manager_arc.lock().await;
                        let _ = manager.write(&bytes);
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "terminal-container",
            style: "width: 100%; height: 100%; background: {scheme.background.to_css()}; overflow: hidden; position: relative; padding: 4px; user-select: none;",
            tabindex: 0,
            onkeydown: handle_keydown,
            onmousedown: handle_mouse_down,
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            onwheel: handle_wheel,

            // Terminal content
            div {
                class: "terminal-content",
                style: "font-family: {font_family}; font-size: {font_size}px; line-height: {cell_height}px; white-space: pre;",

                // Render rows
                for row in 0..current_grid.rows {
                    {render_row(row, &current_grid, &scheme, &current_selection, cell_width)}
                }
            }

            // Cursor overlay
            {render_cursor(&current_grid.cursor, &scheme, cell_width, cell_height)}
        }
    }
}

/// Convert mouse position to cell coordinates
fn mouse_to_cell(e: &MouseEvent, cell_width: f64, cell_height: f64) -> (u16, u16) {
    let coords = e.element_coordinates();
    let x = (coords.x - 4.0).max(0.0); // Subtract padding
    let y = coords.y.max(0.0);

    let col = (x / cell_width) as u16;
    let row = (y / cell_height) as u16;

    (row, col)
}

/// Render a single row
fn render_row(
    row: usize,
    grid: &TerminalGrid,
    scheme: &ColorScheme,
    selection: &Selection,
    cell_width: f64,
) -> Element {
    let row_cells: Vec<&CellInfo> = grid
        .cells
        .iter()
        .filter(|c| c.row as usize == row)
        .collect();

    rsx! {
        div {
            class: "terminal-row",
            style: "display: flex;",

            for col in 0..grid.cols {
                {
                    let is_selected = selection.contains(row as u16, col as u16);
                    if let Some(cell) = row_cells.iter().find(|c| c.col as usize == col) {
                        render_cell(cell, scheme, cell_width, is_selected)
                    } else {
                        render_empty_cell(scheme, cell_width, is_selected)
                    }
                }
            }
        }
    }
}

/// Render a single cell
fn render_cell(cell: &CellInfo, scheme: &ColorScheme, width: f64, selected: bool) -> Element {
    let (fg, bg) = if selected {
        (&scheme.selection_fg, &scheme.selection_bg)
    } else if cell.flags.inverse {
        (&cell.bg, &cell.fg)
    } else {
        (&cell.fg, &cell.bg)
    };

    let mut style = format!(
        "width: {}px; color: {}; background: {};",
        width,
        fg.to_css(),
        bg.to_css()
    );

    if cell.flags.bold {
        style.push_str(" font-weight: bold;");
    }
    if cell.flags.italic {
        style.push_str(" font-style: italic;");
    }
    if cell.flags.underline {
        style.push_str(" text-decoration: underline;");
    }
    if cell.flags.strikethrough {
        style.push_str(" text-decoration: line-through;");
    }
    if cell.flags.hidden {
        style.push_str(" visibility: hidden;");
    }

    let content = if cell.content.is_empty() || cell.content == "\0" {
        " ".to_string()
    } else {
        cell.content.clone()
    };

    rsx! {
        span {
            style: "{style}",
            "{content}"
        }
    }
}

/// Render an empty cell
fn render_empty_cell(scheme: &ColorScheme, width: f64, selected: bool) -> Element {
    let bg = if selected {
        &scheme.selection_bg
    } else {
        &scheme.background
    };

    rsx! {
        span {
            style: "width: {width}px; background: {bg.to_css()};",
            " "
        }
    }
}

/// Render the cursor
fn render_cursor(
    cursor: &CursorInfo,
    scheme: &ColorScheme,
    cell_width: f64,
    cell_height: f64,
) -> Element {
    if !cursor.visible {
        return rsx! {};
    }

    let left = cursor.col as f64 * cell_width + 4.0;
    let top = cursor.row as f64 * cell_height;

    let cursor_style = match cursor.shape {
        CursorShape::Block => format!(
            "width: {}px; height: {}px; background: {};",
            cell_width,
            cell_height,
            scheme.cursor.to_css()
        ),
        CursorShape::Underline => format!(
            "width: {}px; height: 2px; background: {}; margin-top: {}px;",
            cell_width,
            scheme.cursor.to_css(),
            cell_height - 2.0
        ),
        CursorShape::Beam => format!(
            "width: 2px; height: {}px; background: {};",
            cell_height,
            scheme.cursor.to_css()
        ),
    };

    rsx! {
        div {
            class: "cursor",
            style: "position: absolute; left: {left}px; top: {top}px; {cursor_style} opacity: 0.7; pointer-events: none;",
        }
    }
}

/// Convert a key event to terminal bytes
fn key_to_bytes(key: &Key, modifiers: Modifiers) -> Vec<u8> {
    // Handle Ctrl+key combinations
    if modifiers.ctrl() {
        if let Key::Character(c) = key {
            if let Some(ch) = c.chars().next() {
                if ch.is_ascii_lowercase() {
                    return vec![(ch as u8) - b'a' + 1];
                }
            }
        }
    }

    match key {
        Key::Character(c) => c.as_bytes().to_vec(),
        Key::Enter => vec![b'\r'],
        Key::Backspace => vec![0x7f],
        Key::Tab => vec![b'\t'],
        Key::Escape => vec![0x1b],
        Key::ArrowUp => vec![0x1b, b'[', b'A'],
        Key::ArrowDown => vec![0x1b, b'[', b'B'],
        Key::ArrowRight => vec![0x1b, b'[', b'C'],
        Key::ArrowLeft => vec![0x1b, b'[', b'D'],
        Key::Home => vec![0x1b, b'[', b'H'],
        Key::End => vec![0x1b, b'[', b'F'],
        Key::PageUp => vec![0x1b, b'[', b'5', b'~'],
        Key::PageDown => vec![0x1b, b'[', b'6', b'~'],
        Key::Insert => vec![0x1b, b'[', b'2', b'~'],
        Key::Delete => vec![0x1b, b'[', b'3', b'~'],
        Key::F1 => vec![0x1b, b'O', b'P'],
        Key::F2 => vec![0x1b, b'O', b'Q'],
        Key::F3 => vec![0x1b, b'O', b'R'],
        Key::F4 => vec![0x1b, b'O', b'S'],
        Key::F5 => vec![0x1b, b'[', b'1', b'5', b'~'],
        Key::F6 => vec![0x1b, b'[', b'1', b'7', b'~'],
        Key::F7 => vec![0x1b, b'[', b'1', b'8', b'~'],
        Key::F8 => vec![0x1b, b'[', b'1', b'9', b'~'],
        Key::F9 => vec![0x1b, b'[', b'2', b'0', b'~'],
        Key::F10 => vec![0x1b, b'[', b'2', b'1', b'~'],
        Key::F11 => vec![0x1b, b'[', b'2', b'3', b'~'],
        Key::F12 => vec![0x1b, b'[', b'2', b'4', b'~'],
        _ => vec![],
    }
}
