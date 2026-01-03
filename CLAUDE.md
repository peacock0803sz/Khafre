# Claude Development Guide

## Commit Convention

Use emoji prefix (see `~/.config/git/commit-template`):

- `:sparkles:` - New feature (small)
- `:tada:` - New feature (large) / Initial commit
- `:bug:` - Bug fix
- `:recycle:` - Refactoring
- `:wrench:` - Configuration
- `:snowflake:` - Nix related
- `:pencil:` - Documentation
- `:lock:` - Security
- `:white_check_mark:` - Tests

## Project Structure

```
src/
├── main.rs                 # Entry point
├── app.rs                  # Main App component
├── components/             # UI components
│   ├── terminal/           # Terminal view + selection
│   ├── preview.rs          # Sphinx preview
│   └── layout/             # Split view, pane
├── state/                  # State management + hooks
├── services/               # Business logic
│   ├── terminal/           # PTY + alacritty_terminal
│   ├── sphinx.rs           # Sphinx server management
│   └── config.rs           # Configuration loading
└── types/                  # Type definitions
```

## Architecture Notes

### Terminal Emulation

- **alacritty_terminal**: VT parsing and grid management
- **portable-pty**: PTY session management
- Batch output: 16-33ms intervals (~30fps)
- 10000 lines scrollback

### Rendering

- **Dioxus Desktop**: DOM-based rendering
- Cell-by-cell CSS styling
- Selection highlighting
- Mouse wheel scrollback

### Security

- iframe preview uses sandbox attribute
- Dynamic port allocation for sphinx-autobuild

### Configuration

- Global: `$XDG_CONFIG_HOME/khafre/config.toml`
- Project: `.khafre.toml`

## Testing

```bash
cargo test                  # Run all tests
cargo clippy -- -D warnings # Lint
cargo fmt --check           # Format check
```

## Building

```bash
cargo build                 # Debug build
cargo build --release       # Release build
```
