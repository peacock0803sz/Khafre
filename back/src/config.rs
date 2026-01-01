use crate::color_scheme::{load_theme_file, ColorScheme};
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
    /// フォントファミリー
    #[serde(default)]
    pub font_family: Option<String>,
    /// フォントサイズ
    #[serde(default)]
    pub font_size: Option<u16>,
    /// テーマファイルパス（Alacritty/WindowsTerminal/iTerm2形式）
    #[serde(default)]
    pub theme_file: Option<String>,
    /// インラインカラースキーム（theme_fileより優先）
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
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

impl TerminalConfig {
    /// theme_fileからカラースキームを解決
    /// color_schemeが設定済みの場合はそのまま、
    /// theme_fileが設定されている場合はファイルを読み込んでcolor_schemeに変換
    pub fn resolve_color_scheme(&mut self, base_path: Option<&std::path::Path>) {
        // color_schemeが既に設定されている場合はそのまま
        if self.color_scheme.is_some() {
            return;
        }

        // theme_fileが設定されている場合はファイルを読み込む
        if let Some(ref theme_file) = self.theme_file {
            let theme_path = if let Some(base) = base_path {
                base.join(theme_file)
            } else {
                PathBuf::from(theme_file)
            };

            match load_theme_file(&theme_path) {
                Ok(scheme) => {
                    self.color_scheme = Some(scheme);
                }
                Err(e) => {
                    eprintln!("テーマファイル読み込みエラー: {}", e);
                }
            }
        }
    }
}

impl Config {
    /// XDG_CONFIG_HOME/khafre/config.toml から設定を読み込む
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
    /// XDG_CONFIG_HOME/khafre/config.toml または ~/.config/khafre/config.toml
    fn config_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".config"));

        config_dir.join("khafre").join("config.toml")
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
    #[serde(default)]
    pub font_family: Option<String>,
    #[serde(default)]
    pub font_size: Option<u16>,
    #[serde(default)]
    pub theme_file: Option<String>,
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
}

impl TerminalConfigOverride {
    /// theme_fileからカラースキームを解決
    /// color_schemeが設定済みの場合はそのまま、
    /// theme_fileが設定されている場合はファイルを読み込んでcolor_schemeに変換
    pub fn resolve_color_scheme(&mut self) {
        // color_schemeが既に設定されている場合はそのまま
        if self.color_scheme.is_some() {
            return;
        }

        // theme_fileが設定されている場合はファイルを読み込む
        if let Some(ref theme_file) = self.theme_file {
            // DevConfigのtheme_fileは絶対パスを想定
            let theme_path = PathBuf::from(theme_file);

            match load_theme_file(&theme_path) {
                Ok(scheme) => {
                    self.color_scheme = Some(scheme);
                }
                Err(e) => {
                    eprintln!("テーマファイル読み込みエラー: {}", e);
                }
            }
        }
    }
}

fn default_auto_start_sphinx() -> bool {
    true
}

impl DevConfig {
    /// アプリのルートから.khafre.dev.jsonを読み込む
    /// カレントディレクトリと親ディレクトリを順に探索
    pub fn load() -> Option<Self> {
        let current_dir = std::env::current_dir().ok()?;

        // カレントディレクトリと親ディレクトリを順に探索
        // （Tauri devモードではback/から実行されるため）
        let mut candidates = vec![current_dir.join(".khafre.dev.json")];
        if let Some(parent) = current_dir.parent() {
            candidates.push(parent.join(".khafre.dev.json"));
        }

        for config_path in &candidates {
            if config_path.exists() {
                let content = std::fs::read_to_string(config_path).ok()?;
                return serde_json::from_str(&content).ok();
            }
        }

        None
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

    #[test]
    fn test_dev_config_parse_camel_case() {
        // ユーザーが使用するキャメルケースのJSONをパースできるか確認
        let json_str = r#"
        {
            "projectPath": "/path/to/project",
            "autoStartSphinx": true,
            "config": {
                "terminal": { "shell": "/bin/zsh" }
            }
        }
        "#;

        let dev_config: DevConfig = serde_json::from_str(json_str).unwrap();
        println!("Parsed DevConfig: {:?}", dev_config);

        // config.terminal.shell が正しく読み込まれているか確認
        assert!(
            dev_config.config.is_some(),
            "config should be parsed"
        );
        let config = dev_config.config.unwrap();
        assert!(
            config.terminal.is_some(),
            "terminal config should be parsed"
        );
        assert_eq!(
            config.terminal.unwrap().shell,
            Some("/bin/zsh".to_string()),
            "shell should be /bin/zsh"
        );
    }

    #[test]
    fn test_parse_terminal_font_config_toml() {
        // TOMLでフォント設定がパースできるか確認
        let toml_str = r#"
            [sphinx]
            source_dir = "docs"

            [terminal]
            shell = "/bin/zsh"
            font_family = "JetBrains Mono"
            font_size = 16
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.terminal.shell, Some("/bin/zsh".to_string()));
        assert_eq!(config.terminal.font_family, Some("JetBrains Mono".to_string()));
        assert_eq!(config.terminal.font_size, Some(16));
    }

    #[test]
    fn test_parse_terminal_font_config_json() {
        // JSONでフォント設定がパースできるか確認
        let json_str = r#"
        {
            "project_path": "/path/to/project",
            "config": {
                "terminal": {
                    "shell": "/bin/zsh",
                    "font_family": "Fira Code",
                    "font_size": 18
                }
            }
        }
        "#;
        let dev_config: DevConfig = serde_json::from_str(json_str).unwrap();
        let config = dev_config.config.unwrap();
        let terminal = config.terminal.unwrap();
        assert_eq!(terminal.shell, Some("/bin/zsh".to_string()));
        assert_eq!(terminal.font_family, Some("Fira Code".to_string()));
        assert_eq!(terminal.font_size, Some(18));
    }
}
