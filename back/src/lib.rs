mod config;
mod terminal;

use config::Config;
use tauri::State;
use terminal::{create_terminal_manager, SharedTerminalManager};

/// PTYセッションを生成
#[tauri::command]
fn spawn_terminal(
    session_id: String,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    manager: State<'_, SharedTerminalManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.spawn(session_id, cwd, cols, rows, app_handle)
}

/// PTYにデータを書き込む
#[tauri::command]
fn pty_write(
    session_id: String,
    data: String,
    manager: State<'_, SharedTerminalManager>,
) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.write(&session_id, data.as_bytes())
}

/// PTYのサイズを変更
#[tauri::command]
fn pty_resize(
    session_id: String,
    cols: u16,
    rows: u16,
    manager: State<'_, SharedTerminalManager>,
) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.resize(&session_id, cols, rows)
}

/// PTYセッションを終了
#[tauri::command]
fn kill_terminal(
    session_id: String,
    manager: State<'_, SharedTerminalManager>,
) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.kill(&session_id)
}

/// プロジェクト設定を読み込む
#[tauri::command]
fn load_project_config(path: String) -> Result<Config, String> {
    let project_path = std::path::Path::new(&path);
    Config::load(project_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let terminal_manager = create_terminal_manager();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(terminal_manager)
        .invoke_handler(tauri::generate_handler![
            spawn_terminal,
            pty_write,
            pty_resize,
            kill_terminal,
            load_project_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
