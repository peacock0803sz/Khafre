//! Terminal manager integrating alacritty_terminal and PTY
//!
//! This module provides the core terminal emulation functionality
//! by combining alacritty_terminal for VT parsing and grid management
//! with portable-pty for PTY session handling.

use std::io::{Read, Write};
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config as TermConfig, Term};
use alacritty_terminal::tty::{Options as TtyOptions, Pty};
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};
use anyhow::Result;
use tokio::sync::mpsc;

use crate::types::color_scheme::{ColorScheme, Rgb};
use crate::types::terminal::{CellFlags, CellInfo, CursorInfo, CursorShape, TerminalEvent, TerminalGrid};

/// Event listener that forwards events to a channel
pub struct ChannelEventListener {
    tx: mpsc::UnboundedSender<TerminalEvent>,
}

impl EventListener for ChannelEventListener {
    fn send_event(&self, event: Event) {
        let terminal_event = match event {
            Event::Title(title) => Some(TerminalEvent::Title(title)),
            Event::Bell => Some(TerminalEvent::Bell),
            Event::Exit => Some(TerminalEvent::Exit),
            Event::ClipboardStore(_, data) => Some(TerminalEvent::ClipboardStore(data)),
            Event::ClipboardLoad(_, _) => Some(TerminalEvent::ClipboardLoad),
            _ => None,
        };

        if let Some(evt) = terminal_event {
            let _ = self.tx.send(evt);
        }
    }
}

/// Terminal manager handling VT emulation and PTY
pub struct TerminalManager {
    /// Terminal state (grid, cursor, etc.)
    term: Arc<FairMutex<Term<ChannelEventListener>>>,

    /// PTY handle
    pty: Pty,

    /// Event loop notifier for sending data to PTY
    notifier: Notifier,

    /// Event receiver
    event_rx: mpsc::UnboundedReceiver<TerminalEvent>,

    /// Color scheme
    color_scheme: ColorScheme,

    /// Terminal size
    cols: u16,
    rows: u16,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(
        cols: u16,
        rows: u16,
        shell: Option<&str>,
        working_directory: Option<&str>,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let event_listener = ChannelEventListener { tx: event_tx };

        // Terminal configuration
        let term_config = TermConfig::default();

        // Create terminal size
        let size = TermSize::new(cols as usize, rows as usize);
        let window_size = WindowSize {
            num_cols: cols,
            num_lines: rows,
            cell_width: 1,
            cell_height: 1,
        };

        // Create terminal
        let term = Term::new(term_config, &size, event_listener);

        // PTY options
        let tty_options = TtyOptions {
            shell: shell.map(|s| s.into()),
            working_directory: working_directory.map(|s| s.into()),
            hold: false,
        };

        // Create PTY
        let pty = alacritty_terminal::tty::new(&tty_options, window_size, 0)?;

        // Create event loop
        let term = Arc::new(FairMutex::new(term));
        let pty_event_loop = EventLoop::new(
            Arc::clone(&term),
            pty.try_clone()?,
            false,
            false,
        )?;

        let notifier = Notifier(pty_event_loop.channel());

        // Spawn event loop in background thread
        let _event_loop_handle = pty_event_loop.spawn();

        Ok(Self {
            term,
            pty,
            notifier,
            event_rx,
            color_scheme: ColorScheme::default(),
            cols,
            rows,
        })
    }

    /// Write input data to the terminal
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.notifier.0.send(Msg::Input(data.into()))?;
        Ok(())
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;

        let size = TermSize::new(cols as usize, rows as usize);
        let window_size = WindowSize {
            num_cols: cols,
            num_lines: rows,
            cell_width: 1,
            cell_height: 1,
        };

        // Resize terminal
        self.term.lock().resize(size);

        // Resize PTY
        self.notifier.0.send(Msg::Resize(window_size))?;

        Ok(())
    }

    /// Get the current terminal grid for rendering
    pub fn get_grid(&self) -> TerminalGrid {
        let term = self.term.lock();
        let content = term.renderable_content();

        let mut cells = Vec::new();

        for cell in content.display_iter {
            let point = cell.point;
            let cell_data = &cell.cell;

            // Convert colors
            let fg = self.resolve_color(cell_data.fg);
            let bg = self.resolve_color(cell_data.bg);

            // Convert flags
            let flags = CellFlags {
                bold: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::BOLD),
                italic: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::ITALIC),
                underline: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::UNDERLINE),
                strikethrough: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::STRIKEOUT),
                inverse: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::INVERSE),
                hidden: cell_data.flags.contains(alacritty_terminal::term::cell::Flags::HIDDEN),
            };

            cells.push(CellInfo {
                row: point.line.0 as u16,
                col: point.column.0 as u16,
                content: cell_data.c.to_string(),
                fg,
                bg,
                flags,
            });
        }

        // Get cursor info
        let cursor = content.cursor;
        let cursor_info = CursorInfo {
            row: cursor.point.line.0 as u16,
            col: cursor.point.column.0 as u16,
            visible: true, // TODO: handle cursor visibility properly
            shape: match content.cursor.shape {
                alacritty_terminal::vte::ansi::CursorShape::Block => CursorShape::Block,
                alacritty_terminal::vte::ansi::CursorShape::Underline => CursorShape::Underline,
                alacritty_terminal::vte::ansi::CursorShape::Beam => CursorShape::Beam,
            },
        };

        TerminalGrid {
            cells,
            cursor: cursor_info,
            cols: self.cols as usize,
            rows: self.rows as usize,
        }
    }

    /// Resolve an alacritty color to RGB
    fn resolve_color(&self, color: AnsiColor) -> Rgb {
        match color {
            AnsiColor::Named(named) => {
                let index = match named {
                    NamedColor::Black => 0,
                    NamedColor::Red => 1,
                    NamedColor::Green => 2,
                    NamedColor::Yellow => 3,
                    NamedColor::Blue => 4,
                    NamedColor::Magenta => 5,
                    NamedColor::Cyan => 6,
                    NamedColor::White => 7,
                    NamedColor::BrightBlack => 8,
                    NamedColor::BrightRed => 9,
                    NamedColor::BrightGreen => 10,
                    NamedColor::BrightYellow => 11,
                    NamedColor::BrightBlue => 12,
                    NamedColor::BrightMagenta => 13,
                    NamedColor::BrightCyan => 14,
                    NamedColor::BrightWhite => 15,
                    NamedColor::Foreground => return self.color_scheme.foreground,
                    NamedColor::Background => return self.color_scheme.background,
                    NamedColor::Cursor => return self.color_scheme.cursor,
                    _ => return self.color_scheme.foreground, // Default for other named colors
                };
                self.color_scheme.ansi[index]
            }
            AnsiColor::Spec(rgb) => Rgb::new(rgb.r, rgb.g, rgb.b),
            AnsiColor::Indexed(index) => self.color_scheme.resolve_ansi(index),
        }
    }

    /// Set the color scheme
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Get pending terminal events
    pub fn poll_events(&mut self) -> Vec<TerminalEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Scroll the display
    pub fn scroll(&self, delta: i32) {
        use alacritty_terminal::grid::Scroll;
        let mut term = self.term.lock();
        term.scroll_display(Scroll::Delta(delta));
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&self) {
        use alacritty_terminal::grid::Scroll;
        let mut term = self.term.lock();
        term.scroll_display(Scroll::Bottom);
    }

    /// Get terminal dimensions
    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }
}
