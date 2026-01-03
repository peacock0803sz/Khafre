//! Color scheme definitions

use serde::{Deserialize, Serialize};

/// RGB color
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Create a new RGB color
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to CSS rgb() string
    pub fn to_css(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }

    /// Convert to CSS hex string
    #[allow(dead_code)]
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Terminal color scheme
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Background color
    pub background: Rgb,

    /// Foreground color
    pub foreground: Rgb,

    /// Cursor color
    pub cursor: Rgb,

    /// Selection background color
    pub selection_bg: Rgb,

    /// Selection foreground color
    pub selection_fg: Rgb,

    /// 16 ANSI colors (black, red, green, yellow, blue, magenta, cyan, white, + bright variants)
    pub ansi: [Rgb; 16],
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorScheme {
    /// Dark theme (similar to VS Code Dark+)
    pub fn dark() -> Self {
        Self {
            background: Rgb::new(30, 30, 30),       // #1e1e1e
            foreground: Rgb::new(212, 212, 212),    // #d4d4d4
            cursor: Rgb::new(212, 212, 212),        // #d4d4d4
            selection_bg: Rgb::new(38, 79, 120),    // #264f78
            selection_fg: Rgb::new(212, 212, 212),  // #d4d4d4
            ansi: [
                // Normal colors
                Rgb::new(0, 0, 0),         // Black
                Rgb::new(205, 49, 49),     // Red
                Rgb::new(13, 188, 121),    // Green
                Rgb::new(229, 229, 16),    // Yellow
                Rgb::new(36, 114, 200),    // Blue
                Rgb::new(188, 63, 188),    // Magenta
                Rgb::new(17, 168, 205),    // Cyan
                Rgb::new(229, 229, 229),   // White
                // Bright colors
                Rgb::new(102, 102, 102),   // Bright Black
                Rgb::new(241, 76, 76),     // Bright Red
                Rgb::new(35, 209, 139),    // Bright Green
                Rgb::new(245, 245, 67),    // Bright Yellow
                Rgb::new(59, 142, 234),    // Bright Blue
                Rgb::new(214, 112, 214),   // Bright Magenta
                Rgb::new(41, 184, 219),    // Bright Cyan
                Rgb::new(255, 255, 255),   // Bright White
            ],
        }
    }

    /// Light theme (similar to VS Code Light+)
    pub fn light() -> Self {
        Self {
            background: Rgb::new(255, 255, 255),    // #ffffff
            foreground: Rgb::new(0, 0, 0),          // #000000
            cursor: Rgb::new(0, 0, 0),              // #000000
            selection_bg: Rgb::new(173, 214, 255),  // #add6ff
            selection_fg: Rgb::new(0, 0, 0),        // #000000
            ansi: [
                // Normal colors
                Rgb::new(0, 0, 0),         // Black
                Rgb::new(205, 49, 49),     // Red
                Rgb::new(0, 135, 0),       // Green
                Rgb::new(128, 128, 0),     // Yellow
                Rgb::new(0, 0, 128),       // Blue
                Rgb::new(128, 0, 128),     // Magenta
                Rgb::new(0, 135, 135),     // Cyan
                Rgb::new(128, 128, 128),   // White
                // Bright colors
                Rgb::new(102, 102, 102),   // Bright Black
                Rgb::new(241, 76, 76),     // Bright Red
                Rgb::new(0, 175, 0),       // Bright Green
                Rgb::new(175, 135, 0),     // Bright Yellow
                Rgb::new(36, 114, 200),    // Bright Blue
                Rgb::new(175, 0, 175),     // Bright Magenta
                Rgb::new(0, 175, 175),     // Bright Cyan
                Rgb::new(255, 255, 255),   // Bright White
            ],
        }
    }

    /// Resolve an ANSI color index to RGB
    pub fn resolve_ansi(&self, index: u8) -> Rgb {
        if index < 16 {
            self.ansi[index as usize]
        } else if index < 232 {
            // 216 color cube (indices 16-231)
            self.compute_cube_color(index - 16)
        } else {
            // Grayscale (indices 232-255)
            self.compute_grayscale(index - 232)
        }
    }

    /// Compute color from 216 color cube
    fn compute_cube_color(&self, index: u8) -> Rgb {
        let r = (index / 36) * 51;
        let g = ((index / 6) % 6) * 51;
        let b = (index % 6) * 51;
        Rgb::new(r, g, b)
    }

    /// Compute grayscale color
    fn compute_grayscale(&self, index: u8) -> Rgb {
        let gray = index * 10 + 8;
        Rgb::new(gray, gray, gray)
    }
}

/// Theme preference
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}
