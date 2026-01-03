# Contributing to Khafre

## Technology Stack

- **Framework**: Dioxus (Native desktop with WGPU)
- **Language**: Rust
- **Terminal**: alacritty_terminal + portable-pty
- **Async Runtime**: Tokio

### PTY Management

- Use portable-pty (Rust)
- Batch output: 16-33ms intervals
- Throttle resize events during drag

### Security

- iframe preview uses sandbox attribute
- Dynamic port allocation for sphinx-autobuild

## Project Structure

```
khafre/
├── Cargo.toml          # Rust dependencies
├── Dioxus.toml         # Dioxus configuration
├── src/
│   ├── main.rs         # Entry point
│   ├── app.rs          # Main App component
│   ├── components/
│   │   ├── terminal/   # Terminal view + selection
│   │   ├── preview.rs  # Sphinx preview iframe
│   │   └── layout/     # Split pane layout
│   ├── services/
│   │   ├── terminal/   # PTY + alacritty_terminal
│   │   ├── sphinx.rs   # sphinx-autobuild manager
│   │   └── config.rs   # Configuration loader
│   ├── state/          # Dioxus state management
│   └── types/          # Type definitions
└── config.toml.example
```

## Development

### Prerequisites

- Rust (cargo, rustc)
- Dioxus CLI (`cargo install dioxus-cli`)
- System dependencies:
  - **Linux**: gtk3, webkitgtk, libsoup3, glib
  - **macOS**: Xcode Command Line Tools

### Setup

```bash
# With Nix
nix develop

# Or install Dioxus CLI manually
cargo install dioxus-cli
```

### Commands

```bash
dx serve        # Start development server
dx build        # Build for production
cargo test      # Run tests
cargo clippy    # Run linter
cargo fmt       # Format code
```

## Testing

- Use standard `#[cfg(test)]` modules
- Place tests in the same file as implementation
- Example: see `src/components/terminal/selection.rs`

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
