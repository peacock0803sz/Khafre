//! Terminal manager integrating alacritty_terminal and PTY
//!
//! This module provides the core terminal emulation functionality
//! by combining alacritty_terminal for VT parsing and grid management
//! with portable-pty for PTY session handling.

use std::io::{Read, Write};
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config as TermConfig, Term};
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor, Processor};
use anyhow::Result;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
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

    /// PTY writer
    pty_writer: Box<dyn Write + Send>,

    /// PTY reader (wrapped for async reading)
    pty_reader: Option<Box<dyn Read + Send>>,

    /// Event receiver
    event_rx: mpsc::UnboundedReceiver<TerminalEvent>,

    /// Color scheme
    color_scheme: ColorScheme,

    /// Terminal size
    cols: u16,
    rows: u16,

    /// PTY pair (kept alive)
    _pty_pair: portable_pty::PtyPair,
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

        // Create terminal
        let term = Term::new(term_config, &size, event_listener);
        let term = Arc::new(FairMutex::new(term));

        // Create PTY
        let pty_system = NativePtySystem::default();
        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = pty_system.openpty(pty_size)?;

        // Build command
        let shell_cmd = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()).leak()
        });

        let mut cmd = CommandBuilder::new(shell_cmd);
        if let Some(dir) = working_directory {
            cmd.cwd(dir);
        }

        // Spawn shell
        let _child = pty_pair.slave.spawn_command(cmd)?;

        // Get writer and reader
        let pty_writer = pty_pair.master.take_writer()?;
        let pty_reader = Some(pty_pair.master.try_clone_reader()?);

        // Start reading from PTY in background
        let term_clone = Arc::clone(&term);
        let mut reader = pty_pair.master.try_clone_reader()?;

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut processor = Processor::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let mut term = term_clone.lock();
                        processor.advance(&mut *term, &buf[..n]);
                    }
                    Err(e) => {
                        log::error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            term,
            pty_writer,
            pty_reader,
            event_rx,
            color_scheme: ColorScheme::default(),
            cols,
            rows,
            _pty_pair: pty_pair,
        })
    }

    /// Write input data to the terminal
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.pty_writer.write_all(data)?;
        self.pty_writer.flush()?;
        Ok(())
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;

        let size = TermSize::new(cols as usize, rows as usize);

        // Resize terminal
        self.term.lock().resize(size);

        // Resize PTY
        self._pty_pair.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

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
            visible: true,
            shape: match content.cursor.shape {
                alacritty_terminal::vte::ansi::CursorShape::Block => CursorShape::Block,
                alacritty_terminal::vte::ansi::CursorShape::Underline => CursorShape::Underline,
                alacritty_terminal::vte::ansi::CursorShape::Beam => CursorShape::Beam,
                alacritty_terminal::vte::ansi::CursorShape::HollowBlock => CursorShape::Block,
                alacritty_terminal::vte::ansi::CursorShape::Hidden => {
                    return TerminalGrid {
                        cells,
                        cursor: CursorInfo {
                            row: cursor.point.line.0 as u16,
                            col: cursor.point.column.0 as u16,
                            visible: false,
                            shape: CursorShape::Block,
                        },
                        cols: self.cols as usize,
                        rows: self.rows as usize,
                    };
                }
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
                    _ => return self.color_scheme.foreground,
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
