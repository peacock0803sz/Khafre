# Khafre

Sphinx documentation editor with live preview and embedded terminal.

Built with Rust and [Dioxus](https://dioxuslabs.com/).

## Features

- Live preview with sphinx-autobuild
- Native terminal emulation (alacritty_terminal + portable-pty)
- Split-pane layout (terminal + preview)
- Per-project configuration

## Installation

Download the latest release from the [Releases](https://github.com/peacock0803sz/khafre/releases) page.

### Build from Source

```bash
# Install Dioxus CLI
cargo install dioxus-cli

# Build
dx build --release
```

## Usage

1. Open Khafre
2. Select a Sphinx project directory
3. Edit your documentation with live preview

## Configuration

Global configuration: `~/.config/khafre/config.toml`

```toml
[sphinx]
source_dir = "docs"
build_dir = "_build/html"

[sphinx.server]
port = 0  # 0 = auto-assign

[python]
interpreter = ".venv/bin/python"

[editor]
command = "nvim"

[terminal]
# shell = "/bin/zsh"
# font_family = "JetBrains Mono"
# font_size = 14
```

### Options

| Section | Key | Description |
|---------|-----|-------------|
| `sphinx` | `source_dir` | Sphinx source directory |
| `sphinx` | `build_dir` | Build output directory |
| `sphinx.server` | `port` | Preview server port (0 = auto) |
| `python` | `interpreter` | Python interpreter path |
| `editor` | `command` | Editor command |
| `terminal` | `shell` | Shell path (default: $SHELL) |
| `terminal` | `font_family` | Terminal font |
| `terminal` | `font_size` | Terminal font size |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
