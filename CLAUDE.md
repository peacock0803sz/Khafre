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

## Testing Rules

### Rust (back/)

- Use standard `#[cfg(test)]` modules
- Place tests in the same file as implementation

### TypeScript (app/)

- Use Vitest
- Place test files next to source: `*.test.ts`
- Example: `useSphinx.ts` -> `useSphinx.test.ts`

## Architecture Notes

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
