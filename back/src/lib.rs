mod config;
mod sphinx;
mod terminal;

use config::Config;
use sphinx::{create_sphinx_manager, SharedSphinxManager};
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

/// sphinx-autobuildを起動
#[tauri::command]
fn start_sphinx(
    session_id: String,
    project_path: String,
    source_dir: String,
    build_dir: String,
    python_path: String,
    port: u16,
    manager: State<'_, SharedSphinxManager>,
    app_handle: tauri::AppHandle,
) -> Result<u16, String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.start(
        session_id,
        project_path,
        source_dir,
        build_dir,
        python_path,
        port,
        app_handle,
    )
}

/// sphinx-autobuildを停止
#[tauri::command]
fn stop_sphinx(session_id: String, manager: State<'_, SharedSphinxManager>) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.stop(&session_id)
}

/// sphinxのポートを取得
#[tauri::command]
fn get_sphinx_port(
    session_id: String,
    manager: State<'_, SharedSphinxManager>,
) -> Result<Option<u16>, String> {
    let inner = manager.lock().map_err(|e| e.to_string())?;
    Ok(inner.get_port(&session_id))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let terminal_manager = create_terminal_manager();
    let sphinx_manager = create_sphinx_manager();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(terminal_manager)
        .manage(sphinx_manager)
        .invoke_handler(tauri::generate_handler![
            spawn_terminal,
            pty_write,
            pty_resize,
            kill_terminal,
            load_project_config,
            start_sphinx,
            stop_sphinx,
            get_sphinx_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
