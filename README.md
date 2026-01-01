# Khafre

Sphinx documentation editor with live preview and embedded terminal.

## Features

- Live preview with sphinx-autobuild
- Embedded terminal (Neovim integration)
- Split-pane layout (preview + editor)
- Per-project configuration (`.khafre.toml`)

## Installation

Download the latest release from the [Releases](https://github.com/peacock0803sz/khafre/releases) page.

## Usage

1. Open Khafre
2. Select a Sphinx project directory
3. Edit your documentation with live preview

## Configuration

Place `.khafre.toml` in your project root:

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
```

### Options

| Section | Key | Description |
|---------|-----|-------------|
| `sphinx` | `source_dir` | Sphinx source directory |
| `sphinx` | `build_dir` | Build output directory |
| `sphinx.server` | `port` | Preview server port (0 = auto) |
| `python` | `interpreter` | Python interpreter path |
| `editor` | `command` | Editor command |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
