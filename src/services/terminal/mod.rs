//! Terminal management service
//!
//! This module provides terminal emulation using alacritty_terminal
//! and PTY session management using portable-pty.

mod manager;
mod pty;

pub use manager::TerminalManager;
#[allow(unused_imports)]
pub use pty::PtyManager;
