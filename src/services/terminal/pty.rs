//! PTY session management
//!
//! This module manages pseudo-terminal sessions using portable-pty.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;

use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::sync::Mutex;

/// PTY session
pub struct PtySession {
    /// Master PTY handle
    master: Box<dyn portable_pty::MasterPty + Send>,

    /// Writer to PTY
    writer: Box<dyn Write + Send>,

    /// Reader from PTY
    #[allow(dead_code)]
    reader: Box<dyn Read + Send>,
}

/// PTY manager for handling multiple terminal sessions
pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    /// Create a new PTY manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new PTY session
    pub async fn spawn(
        &self,
        session_id: &str,
        cwd: Option<&str>,
        shell: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<()> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Determine shell to use
        let shell_path = shell
            .map(String::from)
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| "/bin/sh".to_string());

        let mut cmd = CommandBuilder::new(&shell_path);

        // Set working directory if provided
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        // Set environment variables
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Spawn the shell
        let _child = pair.slave.spawn_command(cmd)?;

        // Get writer and reader from master
        let writer = pair.master.take_writer()?;
        let reader = pair.master.try_clone_reader()?;

        let session = PtySession {
            master: pair.master,
            writer,
            reader,
        };

        self.sessions
            .lock()
            .await
            .insert(session_id.to_string(), session);

        log::info!("Spawned PTY session: {}", session_id);
        Ok(())
    }

    /// Write data to a PTY session
    pub async fn write(&self, session_id: &str, data: &[u8]) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.writer.write_all(data)?;
            session.writer.flush()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Resize a PTY session
    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let sessions = self.sessions.lock().await;

        if let Some(session) = sessions.get(session_id) {
            session.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            log::debug!("Resized PTY session {} to {}x{}", session_id, cols, rows);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Kill a PTY session
    pub async fn kill(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        if sessions.remove(session_id).is_some() {
            log::info!("Killed PTY session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
