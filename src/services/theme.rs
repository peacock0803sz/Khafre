//! System theme detection service

use crate::types::color_scheme::{ColorScheme, ThemePreference};

/// Detect system theme preference
pub fn detect_system_theme() -> bool {
    // Check for common environment variables that indicate dark mode

    // GTK theme detection
    if let Ok(theme) = std::env::var("GTK_THEME") {
        if theme.to_lowercase().contains("dark") {
            return true;
        }
    }

    // GNOME color scheme
    if let Ok(scheme) = std::env::var("COLOR_SCHEME") {
        if scheme.to_lowercase().contains("dark") {
            return true;
        }
    }

    // XDG color scheme preference
    #[cfg(target_os = "linux")]
    {
        // Try to read from gsettings
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("prefer-dark") {
                return true;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS dark mode detection via defaults command
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().to_lowercase() == "dark" {
                return true;
            }
        }
    }

    // Default to dark if we can't detect
    true
}

/// Get color scheme based on preference and system detection
pub fn get_color_scheme(preference: ThemePreference) -> ColorScheme {
    match preference {
        ThemePreference::Light => ColorScheme::light(),
        ThemePreference::Dark => ColorScheme::dark(),
        ThemePreference::System => {
            if detect_system_theme() {
                ColorScheme::dark()
            } else {
                ColorScheme::light()
            }
        }
    }
}
