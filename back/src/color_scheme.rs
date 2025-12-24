//! カラースキーム定義と各フォーマットのパーサー
//!
//! 対応フォーマット:
//! - Alacritty (TOML)
//! - Windows Terminal (JSON)
//! - iTerm2 (.itermcolors plist)

use serde::{Deserialize, Serialize};
use std::path::Path;

/// xterm.js ITheme互換のカラースキーム
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorScheme {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub cursor_accent: Option<String>,
    #[serde(default)]
    pub selection_background: Option<String>,
    #[serde(default)]
    pub selection_foreground: Option<String>,
    // ANSI colors (0-7)
    #[serde(default)]
    pub black: Option<String>,
    #[serde(default)]
    pub red: Option<String>,
    #[serde(default)]
    pub green: Option<String>,
    #[serde(default)]
    pub yellow: Option<String>,
    #[serde(default)]
    pub blue: Option<String>,
    #[serde(default)]
    pub magenta: Option<String>,
    #[serde(default)]
    pub cyan: Option<String>,
    #[serde(default)]
    pub white: Option<String>,
    // Bright ANSI colors (8-15)
    #[serde(default)]
    pub bright_black: Option<String>,
    #[serde(default)]
    pub bright_red: Option<String>,
    #[serde(default)]
    pub bright_green: Option<String>,
    #[serde(default)]
    pub bright_yellow: Option<String>,
    #[serde(default)]
    pub bright_blue: Option<String>,
    #[serde(default)]
    pub bright_magenta: Option<String>,
    #[serde(default)]
    pub bright_cyan: Option<String>,
    #[serde(default)]
    pub bright_white: Option<String>,
}

/// テーマファイルを読み込み、フォーマットを拡張子から自動検出
pub fn load_theme_file(path: &Path) -> Result<ColorScheme, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("テーマファイル読み込み失敗: {}", e))?;

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "toml" => parse_alacritty_toml(&content),
        "json" => parse_windows_terminal_json(&content),
        "itermcolors" => parse_iterm2_plist(&content),
        _ => Err(format!(
            "未対応のテーマファイル形式: .{} (対応: .toml, .json, .itermcolors)",
            extension
        )),
    }
}

/// Alacritty TOML形式をパース
fn parse_alacritty_toml(content: &str) -> Result<ColorScheme, String> {
    #[derive(Deserialize)]
    struct AlacrittyTheme {
        colors: Option<AlacrittyColors>,
    }

    #[derive(Default, Deserialize)]
    struct AlacrittyColors {
        primary: Option<AlacrittyPrimary>,
        cursor: Option<AlacrityCursor>,
        selection: Option<AlacrittySelection>,
        normal: Option<AlacrittyAnsi>,
        bright: Option<AlacrittyAnsi>,
    }

    #[derive(Default, Deserialize)]
    struct AlacrittyPrimary {
        background: Option<String>,
        foreground: Option<String>,
    }

    #[derive(Default, Deserialize)]
    struct AlacrityCursor {
        cursor: Option<String>,
        text: Option<String>,
    }

    #[derive(Default, Deserialize)]
    struct AlacrittySelection {
        background: Option<String>,
        text: Option<String>,
    }

    #[derive(Default, Deserialize)]
    struct AlacrittyAnsi {
        black: Option<String>,
        red: Option<String>,
        green: Option<String>,
        yellow: Option<String>,
        blue: Option<String>,
        magenta: Option<String>,
        cyan: Option<String>,
        white: Option<String>,
    }

    let theme: AlacrittyTheme =
        toml::from_str(content).map_err(|e| format!("Alacritty TOML パース失敗: {}", e))?;

    let colors = theme.colors.unwrap_or_default();
    let primary = colors.primary.unwrap_or_default();
    let cursor = colors.cursor.unwrap_or_default();
    let selection = colors.selection.unwrap_or_default();
    let normal = colors.normal.unwrap_or_default();
    let bright = colors.bright.unwrap_or_default();

    Ok(ColorScheme {
        background: primary.background,
        foreground: primary.foreground,
        cursor: cursor.cursor,
        cursor_accent: cursor.text,
        selection_background: selection.background,
        selection_foreground: selection.text,
        black: normal.black,
        red: normal.red,
        green: normal.green,
        yellow: normal.yellow,
        blue: normal.blue,
        magenta: normal.magenta,
        cyan: normal.cyan,
        white: normal.white,
        bright_black: bright.black,
        bright_red: bright.red,
        bright_green: bright.green,
        bright_yellow: bright.yellow,
        bright_blue: bright.blue,
        bright_magenta: bright.magenta,
        bright_cyan: bright.cyan,
        bright_white: bright.white,
    })
}

/// Windows Terminal JSON形式をパース
fn parse_windows_terminal_json(content: &str) -> Result<ColorScheme, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WinTermTheme {
        background: Option<String>,
        foreground: Option<String>,
        cursor_color: Option<String>,
        selection_background: Option<String>,
        black: Option<String>,
        red: Option<String>,
        green: Option<String>,
        yellow: Option<String>,
        blue: Option<String>,
        purple: Option<String>,
        cyan: Option<String>,
        white: Option<String>,
        bright_black: Option<String>,
        bright_red: Option<String>,
        bright_green: Option<String>,
        bright_yellow: Option<String>,
        bright_blue: Option<String>,
        bright_purple: Option<String>,
        bright_cyan: Option<String>,
        bright_white: Option<String>,
    }

    let theme: WinTermTheme = serde_json::from_str(content)
        .map_err(|e| format!("Windows Terminal JSON パース失敗: {}", e))?;

    Ok(ColorScheme {
        background: theme.background,
        foreground: theme.foreground,
        cursor: theme.cursor_color,
        cursor_accent: None,
        selection_background: theme.selection_background,
        selection_foreground: None,
        black: theme.black,
        red: theme.red,
        green: theme.green,
        yellow: theme.yellow,
        blue: theme.blue,
        magenta: theme.purple, // Windows TerminalではpurpleがmagentaM
        cyan: theme.cyan,
        white: theme.white,
        bright_black: theme.bright_black,
        bright_red: theme.bright_red,
        bright_green: theme.bright_green,
        bright_yellow: theme.bright_yellow,
        bright_blue: theme.bright_blue,
        bright_magenta: theme.bright_purple,
        bright_cyan: theme.bright_cyan,
        bright_white: theme.bright_white,
    })
}

/// iTerm2 .itermcolors plist形式をパース
fn parse_iterm2_plist(content: &str) -> Result<ColorScheme, String> {
    use std::collections::HashMap;

    // 簡易的なplist XMLパーサー
    // itermcolorsはXML plist形式で、各色がdict内のRGB floatで表現される

    fn extract_color(content: &str, key: &str) -> Option<String> {
        // keyを含むdict要素を探して、その中のRGB値を抽出
        let key_pattern = format!("<key>{}</key>", key);
        let key_pos = content.find(&key_pattern)?;

        // keyの後のdict要素を探す
        let after_key = &content[key_pos..];
        let dict_start = after_key.find("<dict>")?;
        let dict_end = after_key.find("</dict>")?;
        let dict_content = &after_key[dict_start..dict_end + 7];

        // RGB値を抽出
        let r = extract_component(dict_content, "Red Component")?;
        let g = extract_component(dict_content, "Green Component")?;
        let b = extract_component(dict_content, "Blue Component")?;

        Some(rgb_float_to_hex(r, g, b))
    }

    fn extract_component(dict: &str, component: &str) -> Option<f64> {
        let pattern = format!("<key>{}</key>", component);
        let pos = dict.find(&pattern)?;
        let after = &dict[pos..];

        // <real>...</real> または <integer>...</integer> を探す
        if let Some(real_start) = after.find("<real>") {
            let real_end = after.find("</real>")?;
            let value_str = &after[real_start + 6..real_end];
            value_str.parse().ok()
        } else if let Some(int_start) = after.find("<integer>") {
            let int_end = after.find("</integer>")?;
            let value_str = &after[int_start + 9..int_end];
            value_str.parse::<i64>().ok().map(|v| v as f64)
        } else {
            None
        }
    }

    fn rgb_float_to_hex(r: f64, g: f64, b: f64) -> String {
        let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    // iTerm2の色名マッピング
    let color_map: HashMap<&str, &str> = [
        ("Background Color", "background"),
        ("Foreground Color", "foreground"),
        ("Cursor Color", "cursor"),
        ("Cursor Text Color", "cursor_accent"),
        ("Selection Color", "selection_background"),
        ("Selected Text Color", "selection_foreground"),
        ("Ansi 0 Color", "black"),
        ("Ansi 1 Color", "red"),
        ("Ansi 2 Color", "green"),
        ("Ansi 3 Color", "yellow"),
        ("Ansi 4 Color", "blue"),
        ("Ansi 5 Color", "magenta"),
        ("Ansi 6 Color", "cyan"),
        ("Ansi 7 Color", "white"),
        ("Ansi 8 Color", "bright_black"),
        ("Ansi 9 Color", "bright_red"),
        ("Ansi 10 Color", "bright_green"),
        ("Ansi 11 Color", "bright_yellow"),
        ("Ansi 12 Color", "bright_blue"),
        ("Ansi 13 Color", "bright_magenta"),
        ("Ansi 14 Color", "bright_cyan"),
        ("Ansi 15 Color", "bright_white"),
    ]
    .into_iter()
    .collect();

    let mut scheme = ColorScheme::default();

    for (iterm_key, field) in &color_map {
        if let Some(hex) = extract_color(content, iterm_key) {
            match *field {
                "background" => scheme.background = Some(hex),
                "foreground" => scheme.foreground = Some(hex),
                "cursor" => scheme.cursor = Some(hex),
                "cursor_accent" => scheme.cursor_accent = Some(hex),
                "selection_background" => scheme.selection_background = Some(hex),
                "selection_foreground" => scheme.selection_foreground = Some(hex),
                "black" => scheme.black = Some(hex),
                "red" => scheme.red = Some(hex),
                "green" => scheme.green = Some(hex),
                "yellow" => scheme.yellow = Some(hex),
                "blue" => scheme.blue = Some(hex),
                "magenta" => scheme.magenta = Some(hex),
                "cyan" => scheme.cyan = Some(hex),
                "white" => scheme.white = Some(hex),
                "bright_black" => scheme.bright_black = Some(hex),
                "bright_red" => scheme.bright_red = Some(hex),
                "bright_green" => scheme.bright_green = Some(hex),
                "bright_yellow" => scheme.bright_yellow = Some(hex),
                "bright_blue" => scheme.bright_blue = Some(hex),
                "bright_magenta" => scheme.bright_magenta = Some(hex),
                "bright_cyan" => scheme.bright_cyan = Some(hex),
                "bright_white" => scheme.bright_white = Some(hex),
                _ => {}
            }
        }
    }

    Ok(scheme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_alacritty_toml() {
        let toml = r##"
[colors.primary]
background = "#1e1e1e"
foreground = "#d4d4d4"

[colors.normal]
black = "#000000"
red = "#cc0000"
green = "#00cc00"
yellow = "#cccc00"
blue = "#0000cc"
magenta = "#cc00cc"
cyan = "#00cccc"
white = "#cccccc"

[colors.bright]
black = "#555555"
red = "#ff5555"
green = "#55ff55"
yellow = "#ffff55"
blue = "#5555ff"
magenta = "#ff55ff"
cyan = "#55ffff"
white = "#ffffff"
"##;

        let scheme = parse_alacritty_toml(toml).unwrap();
        assert_eq!(scheme.background, Some("#1e1e1e".to_string()));
        assert_eq!(scheme.foreground, Some("#d4d4d4".to_string()));
        assert_eq!(scheme.black, Some("#000000".to_string()));
        assert_eq!(scheme.bright_white, Some("#ffffff".to_string()));
    }

    #[test]
    fn test_parse_windows_terminal_json() {
        let json = r##"
{
    "background": "#1E1E1E",
    "foreground": "#D4D4D4",
    "cursorColor": "#FFFFFF",
    "black": "#000000",
    "red": "#CC0000",
    "green": "#00CC00",
    "yellow": "#CCCC00",
    "blue": "#0000CC",
    "purple": "#CC00CC",
    "cyan": "#00CCCC",
    "white": "#CCCCCC",
    "brightBlack": "#555555",
    "brightRed": "#FF5555",
    "brightGreen": "#55FF55",
    "brightYellow": "#FFFF55",
    "brightBlue": "#5555FF",
    "brightPurple": "#FF55FF",
    "brightCyan": "#55FFFF",
    "brightWhite": "#FFFFFF"
}
"##;

        let scheme = parse_windows_terminal_json(json).unwrap();
        assert_eq!(scheme.background, Some("#1E1E1E".to_string()));
        assert_eq!(scheme.cursor, Some("#FFFFFF".to_string()));
        assert_eq!(scheme.magenta, Some("#CC00CC".to_string()));
        assert_eq!(scheme.bright_magenta, Some("#FF55FF".to_string()));
    }

    #[test]
    fn test_parse_iterm2_plist() {
        let plist = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Background Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.11764705882352941</real>
        <key>Green Component</key>
        <real>0.11764705882352941</real>
        <key>Blue Component</key>
        <real>0.11764705882352941</real>
    </dict>
    <key>Foreground Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.83137254901960789</real>
        <key>Green Component</key>
        <real>0.83137254901960789</real>
        <key>Blue Component</key>
        <real>0.83137254901960789</real>
    </dict>
    <key>Ansi 0 Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.0</real>
        <key>Green Component</key>
        <real>0.0</real>
        <key>Blue Component</key>
        <real>0.0</real>
    </dict>
</dict>
</plist>
"#;

        let scheme = parse_iterm2_plist(plist).unwrap();
        assert_eq!(scheme.background, Some("#1e1e1e".to_string()));
        assert_eq!(scheme.foreground, Some("#d4d4d4".to_string()));
        assert_eq!(scheme.black, Some("#000000".to_string()));
    }

    #[test]
    fn test_rgb_float_to_hex() {
        fn rgb_float_to_hex(r: f64, g: f64, b: f64) -> String {
            let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        }

        assert_eq!(rgb_float_to_hex(0.0, 0.0, 0.0), "#000000");
        assert_eq!(rgb_float_to_hex(1.0, 1.0, 1.0), "#ffffff");
        assert_eq!(rgb_float_to_hex(0.5, 0.5, 0.5), "#808080");
    }
}
