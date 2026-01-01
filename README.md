# Khafre

Sphinx documentation editor with live preview and embedded terminal.

## Features

- Live preview with sphinx-autobuild
- Embedded terminal (Neovim integration)
- Split-pane layout (preview + editor)
- Per-project configuration (`.khafre.toml`)

## Technology Stack

- **Framework**: Tauri v2
- **Backend**: Rust
- **Frontend**: TypeScript + React
- **Terminal**: xterm.js + portable-pty
- **Styling**: Tailwind CSS v4

## Project Structure

```
khafre/
├── back/           # Rust backend (Tauri)
│   ├── src/
│   └── tauri.conf.json
├── app/            # TypeScript frontend (React)
│   ├── components/
│   ├── hooks/
│   └── lib/
├── package.json
└── vite.config.ts
```

## Development

### Prerequisites

- Node.js 22+
- Rust (cargo, rustc)
- Tauri CLI

### Setup

```bash
npm install
npm run tauri dev
```

### Commands

```bash
npm run dev          # Start Vite dev server
npm run tauri dev    # Start Tauri development
npm run build        # Build for production
npm run tauri build  # Build Tauri app
```

## Configuration

Projects using Khafre place `.khafre.toml` in their root:

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
