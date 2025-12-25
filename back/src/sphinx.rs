use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

/// sphinx-autobuildプロセス情報
pub struct SphinxProcess {
    child: Child,
    port: u16,
    /// 停止フラグ（ポーリングスレッド終了用）
    stopped: Arc<AtomicBool>,
}

/// Sphinxプロセスマネージャ
pub struct SphinxManager {
    processes: HashMap<String, SphinxProcess>,
}

impl SphinxManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// 利用可能なポートを検索
    fn find_available_port() -> Result<u16, String> {
        TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("ポートの検索に失敗: {}", e))?
            .local_addr()
            .map_err(|e| format!("アドレスの取得に失敗: {}", e))
            .map(|addr| addr.port())
    }

    /// sphinx-autobuildを起動
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        session_id: String,
        project_path: String,
        source_dir: String,
        build_dir: String,
        python_path: String,
        requested_port: u16,
        extra_args: Vec<String>,
        app_handle: AppHandle,
    ) -> Result<u16, String> {
        // 既存セッションがあれば停止
        if self.processes.contains_key(&session_id) {
            self.stop(&session_id)?;
        }

        let port = if requested_port == 0 {
            Self::find_available_port()?
        } else {
            requested_port
        };

        // python_pathが相対パスの場合、project_pathを基準に解決
        let resolved_python_path = if std::path::Path::new(&python_path).is_relative() {
            let full_path = std::path::Path::new(&project_path).join(&python_path);
            if !full_path.exists() {
                return Err(format!(
                    "Pythonインタプリタが見つかりません: {} (プロジェクト: {})",
                    full_path.display(),
                    project_path
                ));
            }
            full_path.to_string_lossy().to_string()
        } else {
            python_path.clone()
        };

        let source_path = std::path::Path::new(&project_path).join(&source_dir);
        let build_path = std::path::Path::new(&project_path).join(&build_dir);

        // 基本引数を構築
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
        // 追加引数をマージ
        args.extend(extra_args);

        // sphinx-autobuildを起動
        let mut child = Command::new(&resolved_python_path)
            .args(&args)
            .current_dir(&project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "sphinx-autobuildの起動に失敗: {} (Python: {}, 作業ディレクトリ: {})",
                    e, resolved_python_path, project_path
                )
            })?;

        // stderrを監視してビルドイベントを通知
        let stderr = child.stderr.take();
        let sid = session_id.clone();
        let handle = app_handle.clone();

        if let Some(stderr) = stderr {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    // ビルド完了を検出
                    if line.contains("build succeeded") || line.contains("waiting for changes") {
                        let _ = handle.emit("sphinx_built", &sid);
                    }
                    // エラーを検出
                    if line.contains("ERROR") || line.contains("error:") {
                        let _ = handle.emit("sphinx_error", (&sid, &line));
                    }
                }
            });
        }

        // 停止フラグを作成
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_poll = Arc::clone(&stopped);

        // サーバー起動をポーリングで検出（ポートへの接続を試みる）
        let sid_poll = session_id.clone();
        let handle_poll = app_handle.clone();
        let poll_port = port;
        thread::spawn(move || {
            use std::net::TcpStream;
            use std::time::Duration;

            let addr = format!("127.0.0.1:{}", poll_port);
            // 停止されるまで1秒ごとにポーリング
            loop {
                // 停止フラグをチェック
                if stopped_poll.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(Duration::from_secs(1));
                if TcpStream::connect(&addr).is_ok() {
                    let _ = handle_poll.emit("sphinx_started", (&sid_poll, poll_port));
                    return;
                }
            }
        });

        let process = SphinxProcess {
            child,
            port,
            stopped,
        };
        self.processes.insert(session_id.clone(), process);

        Ok(port)
    }

    /// sphinx-autobuildを停止
    pub fn stop(&mut self, session_id: &str) -> Result<(), String> {
        if let Some(mut process) = self.processes.remove(session_id) {
            // ポーリングスレッドに停止を通知
            process.stopped.store(true, Ordering::Relaxed);
            // プロセスをkill
            if let Err(e) = process.child.kill() {
                // 既に終了している場合はエラーを無視
                if e.kind() != std::io::ErrorKind::InvalidInput {
                    return Err(format!("プロセスの停止に失敗: {}", e));
                }
            }
            // 確実に終了を待機（ゾンビプロセス防止）
            let _ = process.child.wait();
        }
        Ok(())
    }

    /// ポートを取得
    pub fn get_port(&self, session_id: &str) -> Option<u16> {
        self.processes.get(session_id).map(|p| p.port)
    }

    /// 実行中かどうか
    #[allow(dead_code)]
    pub fn is_running(&self, session_id: &str) -> bool {
        self.processes.contains_key(session_id)
    }
}

impl Drop for SphinxManager {
    fn drop(&mut self) {
        // 全プロセスを停止
        for (_, mut process) in self.processes.drain() {
            process.stopped.store(true, Ordering::Relaxed);
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }
}

pub type SharedSphinxManager = Arc<Mutex<SphinxManager>>;

pub fn create_sphinx_manager() -> SharedSphinxManager {
    Arc::new(Mutex::new(SphinxManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphinx_manager_creation() {
        let manager = SphinxManager::new();
        assert!(!manager.is_running("test"));
    }

    #[test]
    fn test_find_available_port() {
        let port = SphinxManager::find_available_port().unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_stop_nonexistent_session() {
        let mut manager = SphinxManager::new();
        // 存在しないセッションの停止は成功する
        assert!(manager.stop("nonexistent").is_ok());
    }
}
