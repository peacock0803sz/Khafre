mod color_scheme;
mod config;
mod sphinx;
mod terminal;

use config::{Config, DevConfig};
use sphinx::{create_sphinx_manager, SharedSphinxManager};
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use terminal::{create_terminal_manager, SharedTerminalManager};

/// PTYセッションを生成
#[tauri::command]
fn spawn_terminal(
    session_id: String,
    cwd: Option<String>,
    shell: Option<String>,
    cols: u16,
    rows: u16,
    manager: State<'_, SharedTerminalManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut inner = manager.lock().map_err(|e| e.to_string())?;
    inner.spawn(session_id, cwd, shell, cols, rows, app_handle)
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

/// グローバル設定を読み込む
#[tauri::command]
fn load_config() -> Result<Config, String> {
    let mut config = Config::load()?;
    // テーマファイルがある場合は解決（設定ファイルの場所を基準に）
    let config_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("khafre");
    config.terminal.resolve_color_scheme(Some(&config_dir));
    Ok(config)
}

/// ローカル開発用設定を読み込む
#[tauri::command]
fn load_dev_config() -> Option<DevConfig> {
    let mut config = DevConfig::load()?;
    // テーマファイルがある場合は解決
    if let Some(ref mut terminal) = config.config.as_mut().and_then(|c| c.terminal.as_mut()) {
        terminal.resolve_color_scheme();
    }
    Some(config)
}

/// sphinx-autobuildを起動
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn start_sphinx(
    session_id: String,
    project_path: String,
    source_dir: String,
    build_dir: String,
    python_path: String,
    port: u16,
    extra_args: Vec<String>,
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
        extra_args,
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

/// ブラウザでURLを開く
#[tauri::command]
fn open_in_browser(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
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
            load_config,
            load_dev_config,
            start_sphinx,
            stop_sphinx,
            get_sphinx_port,
            open_in_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
