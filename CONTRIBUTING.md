# Contributing to Khafre

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

## Testing

### Rust (back/)

- Use standard `#[cfg(test)]` modules
- Place tests in the same file as implementation

### TypeScript (app/)

- Use Vitest
- Place test files next to source: `*.test.ts`
- Example: `useSphinx.ts` -> `useSphinx.test.ts`
