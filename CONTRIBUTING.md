# Contributing to Khafre

## Technology Stack

- **Framework**: Tauri v2
- **Backend**: Rust
- **Frontend**: TypeScript + React
- **Terminal**: xterm.js + portable-pty
- **Styling**: Tailwind CSS v4

### PTY Management

- Use portable-pty (Rust)
- Batch output: 16-33ms intervals
- Throttle resize events during drag

### Security

- Session nonce required for all IPC calls (from MVP)
- iframe preview uses sandbox attribute
- Dynamic port allocation for sphinx-autobuild

### xterm.js

- Use CanvasAddon (not WebGL) for WKWebView compatibility
- scrollback limit: 10000 lines

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

## Testing

### Rust (back/)

- Use standard `#[cfg(test)]` modules
- Place tests in the same file as implementation

### TypeScript (app/)

- Use Vitest
- Place test files next to source: `*.test.ts`
- Example: `useSphinx.ts` -> `useSphinx.test.ts`

## Commit Convention

Use emoji prefix (see `./.gitmessage`):

- `:sparkles:` - New feature (small)
- `:tada:` - New feature (large) / Initial commit
- `:bug:` - Bug fix
- `:recycle:` - Refactoring
- `:wrench:` - Configuration
- `:snowflake:` - Nix related
- `:pencil:` - Documentation
- `:lock:` - Security
- `:white_check_mark:` - Tests
