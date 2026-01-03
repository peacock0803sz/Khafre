//! Sphinx documentation server management
//!
//! This module manages sphinx-autobuild processes for live documentation preview.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use tokio::sync::mpsc;

/// Sphinx build event
#[derive(Clone, Debug)]
pub enum SphinxEvent {
    /// Server started on port
    Started { session_id: String, port: u16 },

    /// Build completed
    Built { session_id: String },

    /// Build error
    Error { session_id: String, message: String },

    /// Server stopped
    Stopped { session_id: String },
}

/// sphinx-autobuild process information
struct SphinxProcess {
    child: Child,
    port: u16,
    /// Stop flag for polling thread
    stopped: Arc<AtomicBool>,
}

/// Sphinx process manager
pub struct SphinxManager {
    processes: HashMap<String, SphinxProcess>,
    event_tx: mpsc::UnboundedSender<SphinxEvent>,
}

impl SphinxManager {
    /// Create a new Sphinx manager
    pub fn new() -> (Self, mpsc::UnboundedReceiver<SphinxEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                processes: HashMap::new(),
                event_tx: tx,
            },
            rx,
        )
    }

    /// Find an available port
    fn find_available_port() -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        Ok(port)
    }

    /// Start sphinx-autobuild
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        session_id: String,
        project_path: &str,
        source_dir: &str,
        build_dir: &str,
        python_path: &str,
        requested_port: u16,
        extra_args: Vec<String>,
    ) -> Result<u16> {
        // Stop existing session if any
        if self.processes.contains_key(&session_id) {
            self.stop(&session_id)?;
        }

        let port = if requested_port == 0 {
            Self::find_available_port()?
        } else {
            requested_port
        };

        // Resolve relative python path
        let resolved_python_path = if std::path::Path::new(python_path).is_relative() {
            let full_path = std::path::Path::new(project_path).join(python_path);
            if !full_path.exists() {
                anyhow::bail!(
                    "Python interpreter not found: {} (project: {})",
                    full_path.display(),
                    project_path
                );
            }
            full_path.to_string_lossy().to_string()
        } else {
            python_path.to_string()
        };

        let source_path = std::path::Path::new(project_path).join(source_dir);
        let build_path = std::path::Path::new(project_path).join(build_dir);

        // Build arguments
        let mut args = vec![
            "-m".to_string(),
            "sphinx_autobuild".to_string(),
            source_path.to_str().unwrap().to_string(),
            build_path.to_str().unwrap().to_string(),
            "--port".to_string(),
            port.to_string(),
            "--host".to_string(),
            "127.0.0.1".to_string(),
        ];
        args.extend(extra_args);

        // Start sphinx-autobuild
        let mut child = Command::new(&resolved_python_path)
            .args(&args)
            .current_dir(project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Monitor stderr for build events
        let stderr = child.stderr.take();
        let sid = session_id.clone();
        let event_tx = self.event_tx.clone();

        if let Some(stderr) = stderr {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    // Detect build completion
                    if line.contains("build succeeded") || line.contains("waiting for changes") {
                        let _ = event_tx.send(SphinxEvent::Built {
                            session_id: sid.clone(),
                        });
                    }
                    // Detect errors
                    if line.contains("ERROR") || line.contains("error:") {
                        let _ = event_tx.send(SphinxEvent::Error {
                            session_id: sid.clone(),
                            message: line,
                        });
                    }
                }
            });
        }

        // Create stop flag
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_poll = Arc::clone(&stopped);

        // Poll for server startup
        let sid_poll = session_id.clone();
        let event_tx_poll = self.event_tx.clone();
        let poll_port = port;

        thread::spawn(move || {
            use std::net::TcpStream;
            use std::time::Duration;

            let addr = format!("127.0.0.1:{}", poll_port);

            loop {
                if stopped_poll.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(Duration::from_secs(1));
                if TcpStream::connect(&addr).is_ok() {
                    let _ = event_tx_poll.send(SphinxEvent::Started {
                        session_id: sid_poll.clone(),
                        port: poll_port,
                    });
                    return;
                }
            }
        });

        let process = SphinxProcess {
            child,
            port,
            stopped,
        };
        self.processes.insert(session_id, process);

        Ok(port)
    }

    /// Stop sphinx-autobuild
    pub fn stop(&mut self, session_id: &str) -> Result<()> {
        if let Some(mut process) = self.processes.remove(session_id) {
            // Signal polling thread to stop
            process.stopped.store(true, Ordering::Relaxed);

            // Kill process
            if let Err(e) = process.child.kill() {
                if e.kind() != std::io::ErrorKind::InvalidInput {
                    anyhow::bail!("Failed to stop process: {}", e);
                }
            }

            // Wait for process to exit (prevent zombie)
            let _ = process.child.wait();

            let _ = self.event_tx.send(SphinxEvent::Stopped {
                session_id: session_id.to_string(),
            });
        }
        Ok(())
    }

    /// Get port for a session
    pub fn get_port(&self, session_id: &str) -> Option<u16> {
        self.processes.get(session_id).map(|p| p.port)
    }

    /// Check if a session is running
    pub fn is_running(&self, session_id: &str) -> bool {
        self.processes.contains_key(session_id)
    }
}

impl Drop for SphinxManager {
    fn drop(&mut self) {
        // Stop all processes
        for (_, mut process) in self.processes.drain() {
            process.stopped.store(true, Ordering::Relaxed);
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphinx_manager_creation() {
        let (manager, _rx) = SphinxManager::new();
        assert!(!manager.is_running("test"));
    }

    #[test]
    fn test_find_available_port() {
        let port = SphinxManager::find_available_port().unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_stop_nonexistent_session() {
        let (mut manager, _rx) = SphinxManager::new();
        assert!(manager.stop("nonexistent").is_ok());
    }
}
