//! Configuration types

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Sphinx configuration
    #[serde(default)]
    pub sphinx: SphinxConfig,

    /// Python configuration
    #[serde(default)]
    pub python: PythonConfig,

    /// Editor configuration
    #[serde(default)]
    pub editor: EditorConfig,

    /// Terminal configuration
    #[serde(default)]
    pub terminal: TerminalConfig,
}

/// Sphinx configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SphinxConfig {
    /// Source directory
    #[serde(default = "default_source_dir")]
    pub source_dir: String,

    /// Build directory
    #[serde(default = "default_build_dir")]
    pub build_dir: String,

    /// Server configuration
    #[serde(default)]
    pub server: SphinxServerConfig,

    /// Extra arguments for sphinx-autobuild
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for SphinxConfig {
    fn default() -> Self {
        Self {
            source_dir: default_source_dir(),
            build_dir: default_build_dir(),
            server: SphinxServerConfig::default(),
            extra_args: Vec::new(),
        }
    }
}

fn default_source_dir() -> String {
    "docs".to_string()
}

fn default_build_dir() -> String {
    "_build/html".to_string()
}

/// Sphinx server configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SphinxServerConfig {
    /// Port (0 for auto-assign)
    #[serde(default)]
    pub port: u16,
}

/// Python configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Python interpreter path
    #[serde(default = "default_interpreter")]
    pub interpreter: String,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            interpreter: default_interpreter(),
        }
    }
}

fn default_interpreter() -> String {
    ".venv/bin/python".to_string()
}

/// Editor configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Editor command
    #[serde(default = "default_editor")]
    pub command: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            command: default_editor(),
        }
    }
}

fn default_editor() -> String {
    "nvim".to_string()
}

/// Terminal configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Shell path
    pub shell: Option<String>,

    /// Font family
    #[serde(default = "default_font_family")]
    pub font_family: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Theme file path
    pub theme_file: Option<String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            font_family: default_font_family(),
            font_size: default_font_size(),
            theme_file: None,
        }
    }
}

fn default_font_family() -> String {
    "Menlo".to_string()
}

fn default_font_size() -> u32 {
    14
}
