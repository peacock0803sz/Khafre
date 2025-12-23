use serde::{Deserialize, Serialize};
use std::path::Path;

/// プロジェクト設定全体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub sphinx: SphinxConfig,
    #[serde(default)]
    pub python: PythonConfig,
    #[serde(default)]
    pub editor: EditorConfig,
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
}

/// sphinx-autobuildサーバー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            sphinx: SphinxConfig::default(),
            python: PythonConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

impl Default for SphinxConfig {
    fn default() -> Self {
        Self {
            source_dir: default_source_dir(),
            build_dir: default_build_dir(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 0 }
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
    /// プロジェクトパスから設定を読み込む
    /// .orthrus.tomlが存在しない場合はデフォルト値を返す
    pub fn load(project_path: &Path) -> Result<Self, String> {
        let config_path = project_path.join(".orthrus.toml");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("設定ファイルの読み込みに失敗: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("設定ファイルのパースに失敗: {}", e))
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
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sphinx.source_dir, "docs/source");
        assert_eq!(config.sphinx.build_dir, "docs/_build");
        assert_eq!(config.sphinx.server.port, 8080);
        assert_eq!(config.python.interpreter, ".venv/bin/python");
        assert_eq!(config.editor.command, "vim");
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let path = Path::new("/nonexistent/path");
        let config = Config::load(path).unwrap();
        assert_eq!(config.sphinx.source_dir, "docs");
    }
}
