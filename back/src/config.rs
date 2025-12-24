use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// プロジェクト設定全体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub sphinx: SphinxConfig,
    #[serde(default)]
    pub python: PythonConfig,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
}

/// Sphinx関連設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphinxConfig {
    #[serde(default = "default_source_dir")]
    pub source_dir: String,
    #[serde(default = "default_build_dir")]
    pub build_dir: String,
    #[serde(default)]
    pub server: ServerConfig,
    /// sphinx-autobuild への追加引数
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// sphinx-autobuildサーバー設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub port: u16, // 0 = 自動割り当て
}

/// Python環境設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    #[serde(default = "default_interpreter")]
    pub interpreter: String,
}

/// エディタ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_editor")]
    pub command: String,
}

/// ターミナル設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// シェルパス (None = $SHELL から自動検出)
    #[serde(default)]
    pub shell: Option<String>,
}

// デフォルト値関数
fn default_source_dir() -> String {
    "docs".to_string()
}

fn default_build_dir() -> String {
    "_build/html".to_string()
}

fn default_interpreter() -> String {
    "python".to_string()
}

fn default_editor() -> String {
    "nvim".to_string()
}

impl Default for SphinxConfig {
    fn default() -> Self {
        Self {
            source_dir: default_source_dir(),
            build_dir: default_build_dir(),
            server: ServerConfig::default(),
            extra_args: Vec::new(),
        }
    }
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            interpreter: default_interpreter(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            command: default_editor(),
        }
    }
}

impl Config {
    /// XDG_CONFIG_HOME/orthrus/config.toml から設定を読み込む
    /// 設定ファイルが存在しない場合はデフォルト値を返す
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("設定ファイルの読み込みに失敗: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("設定ファイルのパースに失敗: {}", e))
    }

    /// 設定ファイルのパスを取得
    /// XDG_CONFIG_HOME/orthrus/config.toml または ~/.config/orthrus/config.toml
    fn config_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".config"));

        config_dir.join("orthrus").join("config.toml")
    }
}

/// ローカル開発用設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevConfig {
    /// デフォルトで開くプロジェクトパス
    #[serde(default)]
    pub project_path: Option<String>,
    /// sphinx-autobuildを自動起動するか
    #[serde(default = "default_auto_start_sphinx")]
    pub auto_start_sphinx: bool,
    /// グローバル設定の上書き（部分的に指定可能）
    #[serde(default)]
    pub config: Option<ConfigOverride>,
}

/// 設定の部分上書き用構造体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigOverride {
    #[serde(default)]
    pub sphinx: Option<SphinxConfigOverride>,
    #[serde(default)]
    pub python: Option<PythonConfigOverride>,
    #[serde(default)]
    pub editor: Option<EditorConfigOverride>,
    #[serde(default)]
    pub terminal: Option<TerminalConfigOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SphinxConfigOverride {
    #[serde(default)]
    pub source_dir: Option<String>,
    #[serde(default)]
    pub build_dir: Option<String>,
    #[serde(default)]
    pub server: Option<ServerConfigOverride>,
    #[serde(default)]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfigOverride {
    #[serde(default)]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PythonConfigOverride {
    #[serde(default)]
    pub interpreter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorConfigOverride {
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalConfigOverride {
    #[serde(default)]
    pub shell: Option<String>,
}

fn default_auto_start_sphinx() -> bool {
    true
}

impl DevConfig {
    /// アプリのルートから.orthrus.dev.jsonを読み込む
    pub fn load() -> Option<Self> {
        // カレントディレクトリから探す
        let config_path = std::env::current_dir().ok()?.join(".orthrus.dev.json");

        if !config_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&config_path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.sphinx.source_dir, "docs");
        assert_eq!(config.sphinx.build_dir, "_build/html");
        assert_eq!(config.sphinx.server.port, 0);
        assert_eq!(config.python.interpreter, "python");
        assert_eq!(config.editor.command, "nvim");
        assert!(config.terminal.shell.is_none());
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
            [sphinx]
            source_dir = "source"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sphinx.source_dir, "source");
        // 他のフィールドはデフォルト値
        assert_eq!(config.sphinx.build_dir, "_build/html");
        assert_eq!(config.python.interpreter, "python");
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
            [sphinx]
            source_dir = "docs/source"
            build_dir = "docs/_build"

            [sphinx.server]
            port = 8080

            [python]
            interpreter = ".venv/bin/python"

            [editor]
            command = "vim"

            [terminal]
            shell = "/opt/homebrew/bin/fish"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sphinx.source_dir, "docs/source");
        assert_eq!(config.sphinx.build_dir, "docs/_build");
        assert_eq!(config.sphinx.server.port, 8080);
        assert_eq!(config.python.interpreter, ".venv/bin/python");
        assert_eq!(config.editor.command, "vim");
        assert_eq!(
            config.terminal.shell,
            Some("/opt/homebrew/bin/fish".to_string())
        );
    }

    #[test]
    fn test_load_returns_default_when_no_config() {
        // XDG_CONFIG_HOMEを存在しないパスに設定してテスト
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path/for/test");
        let config = Config::load().unwrap();
        assert_eq!(config.sphinx.source_dir, "docs");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
