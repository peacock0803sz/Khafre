# Tauri → Dioxus Native 移行計画

## 概要

Tauri + React + xterm.js から **Dioxus Native + alacritty_terminal** への完全ネイティブ化移行。

### 目標
- 完全Rust化（TypeScript/JavaScript排除）
- ネイティブターミナルによる高パフォーマンス
- 単一言語による保守性向上
- IPC オーバーヘッドの排除

---

## 現状分析

### 現在のアーキテクチャ
```
┌─────────────────────────────────────────────────────────┐
│                    Tauri App                            │
├───────────────────────┬─────────────────────────────────┤
│   Frontend (app/)     │      Backend (back/)            │
│   React + TypeScript  │      Rust + Tauri Commands      │
│   Vite + Tailwind     │      portable-pty, tokio        │
│   xterm.js            │      sphinx-autobuild管理       │
├───────────────────────┴─────────────────────────────────┤
│                   WebView (WKWebView/WebView2)          │
└─────────────────────────────────────────────────────────┘
```

### 主要コンポーネント
| コンポーネント | 現在 | 移行後 |
|---------------|------|--------|
| UI Framework | React 19 + TypeScript | Dioxus Native (Rust) |
| レンダリング | WebView | WGPU (GPU) |
| ターミナル | xterm.js | alacritty_terminal + カスタム描画 |
| IPC | Tauri Commands | 直接呼び出し |
| PTY管理 | portable-pty | portable-pty (維持) |
| Sphinx管理 | tokio + subprocess | tokio + subprocess (維持) |

---

## 新アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────┐
│                    Khafre (Dioxus Native)                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    UI Layer (RSX)                        │   │
│  │  ┌─────────────────┐  ┌───────────────────────────────┐ │   │
│  │  │ TerminalView    │  │ PreviewPane                   │ │   │
│  │  │ (カスタム描画)   │  │ (WebView埋込 or Native HTML) │ │   │
│  │  └────────┬────────┘  └───────────────────────────────┘ │   │
│  │           │                                              │   │
│  │  ┌────────▼────────────────────────────────────────────┐ │   │
│  │  │              SplitView (ドラッグ対応)                │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                   State Layer (Signals)                  │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌─────────────────┐  │   │
│  │  │ TerminalState│ │ SphinxState  │ │ ConfigState     │  │   │
│  │  │ - grid       │ │ - port       │ │ - theme         │  │   │
│  │  │ - cursor     │ │ - status     │ │ - font          │  │   │
│  │  │ - selection  │ │ - output     │ │ - color_scheme  │  │   │
│  │  └──────────────┘ └──────────────┘ └─────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌───────────────────────────▼─────────────────────────────┐   │
│  │                  Service Layer                           │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │ TerminalManager                                      │ │   │
│  │  │  ├── alacritty_terminal::Term (VTパーシング/グリッド)│ │   │
│  │  │  ├── portable-pty (PTYセッション)                    │ │   │
│  │  │  └── TerminalRenderer (WGPU描画)                     │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │ SphinxManager                                        │ │   │
│  │  │  ├── sphinx-autobuild プロセス管理                   │ │   │
│  │  │  └── ビルドイベント監視                              │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │ ConfigManager                                        │ │   │
│  │  │  ├── TOML設定読み込み                                │ │   │
│  │  │  └── カラースキーム管理                              │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│              Dioxus Native (Blitz + WGPU)                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 依存関係

### Cargo.toml
```toml
[package]
name = "khafre"
version = "0.2.0"
edition = "2024"

[dependencies]
# Dioxus Native
dioxus = { version = "0.6", features = ["native"] }

# ターミナルエミュレーション
alacritty_terminal = "0.25"

# PTY管理
portable-pty = "0.9"

# 非同期ランタイム
tokio = { version = "1", features = ["full"] }

# シリアライズ
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.9"

# ユーティリティ
dirs = "6"
rand = "0.9"
log = "0.4"
env_logger = "0.11"
anyhow = "1"
thiserror = "2"

# フォント処理
fontdue = "0.9"
cosmic-text = "0.12"

# プレビュー用WebView（オプション）
wry = { version = "0.50", optional = true }

[features]
default = ["webview-preview"]
webview-preview = ["wry"]

[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## プロジェクト構造

```
Khafre/
├── src/
│   ├── main.rs                 # エントリーポイント
│   ├── app.rs                  # メインAppコンポーネント
│   │
│   ├── components/             # UIコンポーネント
│   │   ├── mod.rs
│   │   ├── terminal/
│   │   │   ├── mod.rs
│   │   │   ├── view.rs         # TerminalViewコンポーネント
│   │   │   ├── renderer.rs     # WGPU/Canvas描画ロジック
│   │   │   ├── input.rs        # キーボード/マウス入力処理
│   │   │   └── selection.rs    # テキスト選択
│   │   ├── preview.rs          # Sphinxプレビュー
│   │   └── layout/
│   │       ├── mod.rs
│   │       ├── split_view.rs   # 分割ペイン
│   │       └── pane.rs
│   │
│   ├── state/                  # 状態管理
│   │   ├── mod.rs
│   │   ├── terminal.rs         # ターミナル状態
│   │   ├── sphinx.rs           # Sphinx状態
│   │   └── config.rs           # 設定状態
│   │
│   ├── services/               # ビジネスロジック
│   │   ├── mod.rs
│   │   ├── terminal/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs      # TerminalManager
│   │   │   ├── pty.rs          # PTYセッション管理
│   │   │   └── event.rs        # ターミナルイベント処理
│   │   ├── sphinx.rs           # SphinxManager
│   │   └── config.rs           # ConfigManager
│   │
│   └── types/                  # 型定義
│       ├── mod.rs
│       ├── config.rs
│       ├── color_scheme.rs
│       └── terminal.rs
│
├── assets/
│   └── fonts/                  # 組み込みフォント（オプション）
│
├── Cargo.toml
├── Dioxus.toml
└── README.md
```

---

## Phase 1: 基盤構築

### 1.1 プロジェクト初期化

```bash
# 新しいDioxusプロジェクト作成
dx new khafre-native --template native

# または既存プロジェクトを変換
cargo init
```

### 1.2 最小限のDioxus Nativeアプリ

```rust
// src/main.rs
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "display: flex; height: 100vh;",
            div {
                style: "flex: 1; background: #1e1e1e;",
                "Terminal Placeholder"
            }
            div {
                style: "width: 4px; background: #333; cursor: col-resize;"
            }
            div {
                style: "flex: 1; background: #fff;",
                "Preview Placeholder"
            }
        }
    }
}
```

### 1.3 既存サービス層の移行

バックエンド（`back/src/`）から以下を移行:

| 元ファイル | 移行先 | 変更点 |
|-----------|--------|--------|
| `terminal.rs` | `services/terminal/pty.rs` | Tauri依存削除 |
| `sphinx.rs` | `services/sphinx.rs` | Tauri依存削除 |
| `config.rs` | `services/config.rs` | そのまま |
| `color_scheme.rs` | `types/color_scheme.rs` | そのまま |

```rust
// services/terminal/pty.rs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PtySession {
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
    pub reader: Box<dyn std::io::Read + Send>,
}

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(
        &self,
        session_id: &str,
        cwd: &str,
        shell: &str,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<()> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(shell);
        let mut cmd = cmd;
        cmd.cwd(cwd);

        let _child = pair.slave.spawn_command(cmd)?;

        let master = pair.master;
        let writer = master.take_writer()?;
        let reader = master.try_clone_reader()?;

        let session = PtySession {
            master,
            writer,
            reader,
        };

        self.sessions
            .lock()
            .await
            .insert(session_id.to_string(), session);

        Ok(())
    }

    pub async fn write(&self, session_id: &str, data: &[u8]) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.writer.write_all(data)?;
        }
        Ok(())
    }

    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) -> anyhow::Result<()> {
        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(session_id) {
            session.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }
        Ok(())
    }
}
```

---

## Phase 2: alacritty_terminal 統合

### 2.1 ターミナルエミュレーション層

```rust
// services/terminal/manager.rs
use alacritty_terminal::term::{Term, Config as TermConfig};
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::grid::Grid;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::tty::Pty;
use alacritty_terminal::sync::FairMutex;
use std::sync::Arc;

pub struct TerminalEventListener {
    tx: tokio::sync::mpsc::Sender<TerminalEvent>,
}

impl EventListener for TerminalEventListener {
    fn send_event(&self, event: Event) {
        let _ = self.tx.blocking_send(TerminalEvent::from(event));
    }
}

pub struct TerminalManager {
    term: Arc<FairMutex<Term<TerminalEventListener>>>,
    pty: Pty,
    event_rx: tokio::sync::mpsc::Receiver<TerminalEvent>,
}

impl TerminalManager {
    pub fn new(cols: u16, rows: u16) -> anyhow::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let listener = TerminalEventListener { tx };

        let config = TermConfig::default();
        let size = alacritty_terminal::term::SizeInfo::new(
            cols as f32,
            rows as f32,
            1.0, 1.0, 0.0, 0.0, false,
        );

        let term = Term::new(config, &size, listener);

        // PTY生成
        let pty_config = alacritty_terminal::tty::Options {
            shell: None, // デフォルトシェル
            working_directory: None,
            hold: false,
        };
        let pty = alacritty_terminal::tty::new(&pty_config, size.into(), 0)?;

        Ok(Self {
            term: Arc::new(FairMutex::new(term)),
            pty,
            event_rx: rx,
        })
    }

    /// PTYからの出力を処理してTermに反映
    pub fn process_pty_output(&self, data: &[u8]) {
        let mut term = self.term.lock();
        for byte in data {
            term.input(*byte);
        }
    }

    /// グリッドの内容を取得（描画用）
    pub fn get_grid(&self) -> TerminalGrid {
        let term = self.term.lock();
        let grid = term.grid();

        TerminalGrid {
            cells: grid
                .display_iter()
                .map(|indexed| CellInfo {
                    row: indexed.point.line.0 as u16,
                    col: indexed.point.column.0 as u16,
                    content: indexed.cell.c.to_string(),
                    fg: indexed.cell.fg,
                    bg: indexed.cell.bg,
                    flags: indexed.cell.flags,
                })
                .collect(),
            cursor: CursorInfo {
                row: term.grid().cursor.point.line.0 as u16,
                col: term.grid().cursor.point.column.0 as u16,
                visible: !term.cursor_style().blinking || /* blink state */,
            },
            cols: term.columns(),
            rows: term.screen_lines(),
        }
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.pty.writer().write_all(data)?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        let size = alacritty_terminal::term::SizeInfo::new(
            cols as f32, rows as f32, 1.0, 1.0, 0.0, 0.0, false,
        );
        self.term.lock().resize(size);
        let _ = self.pty.resize(size.into());
    }
}

#[derive(Clone)]
pub struct TerminalGrid {
    pub cells: Vec<CellInfo>,
    pub cursor: CursorInfo,
    pub cols: usize,
    pub rows: usize,
}

#[derive(Clone)]
pub struct CellInfo {
    pub row: u16,
    pub col: u16,
    pub content: String,
    pub fg: alacritty_terminal::vte::ansi::Color,
    pub bg: alacritty_terminal::vte::ansi::Color,
    pub flags: alacritty_terminal::term::cell::Flags,
}

#[derive(Clone)]
pub struct CursorInfo {
    pub row: u16,
    pub col: u16,
    pub visible: bool,
}
```

### 2.2 カラー変換

```rust
// types/color_scheme.rs
use alacritty_terminal::vte::ansi::Color as AnsiColor;

#[derive(Clone)]
pub struct ColorScheme {
    pub background: Rgb,
    pub foreground: Rgb,
    pub cursor: Rgb,
    pub selection_bg: Rgb,
    pub selection_fg: Rgb,
    pub ansi: [Rgb; 16],
}

#[derive(Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorScheme {
    pub fn resolve_color(&self, color: AnsiColor) -> Rgb {
        match color {
            AnsiColor::Named(named) => self.ansi[named as usize],
            AnsiColor::Spec(rgb) => Rgb {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
            },
            AnsiColor::Indexed(idx) => {
                if idx < 16 {
                    self.ansi[idx as usize]
                } else {
                    // 256色パレット計算
                    self.compute_indexed_color(idx)
                }
            }
        }
    }

    fn compute_indexed_color(&self, idx: u8) -> Rgb {
        if idx < 232 {
            // 216色キューブ (16-231)
            let idx = idx - 16;
            let r = (idx / 36) * 51;
            let g = ((idx / 6) % 6) * 51;
            let b = (idx % 6) * 51;
            Rgb { r, g, b }
        } else {
            // グレースケール (232-255)
            let gray = (idx - 232) * 10 + 8;
            Rgb { r: gray, g: gray, b: gray }
        }
    }
}
```

---

## Phase 3: ターミナルレンダリング

### 3.1 DOM/CSSベースの描画（初期実装）

```rust
// components/terminal/view.rs
use dioxus::prelude::*;
use crate::services::terminal::TerminalManager;
use crate::types::color_scheme::ColorScheme;

#[component]
pub fn TerminalView(
    session_id: String,
    color_scheme: ColorScheme,
) -> Element {
    let terminal = use_context::<Signal<TerminalManager>>();
    let grid = use_signal(|| None::<TerminalGrid>);

    // PTY出力を監視してグリッドを更新
    use_effect(move || {
        let terminal = terminal.read();
        spawn(async move {
            loop {
                // 16-33ms間隔で更新（60-30fps）
                tokio::time::sleep(Duration::from_millis(16)).await;

                let new_grid = terminal.get_grid();
                grid.set(Some(new_grid));
            }
        });
    });

    let Some(g) = grid.read().clone() else {
        return rsx! { div { "Loading..." } };
    };

    let cell_width = 9.0;  // monospace font width
    let cell_height = 18.0;

    rsx! {
        div {
            class: "terminal-container",
            style: "font-family: 'Menlo', 'Monaco', monospace; font-size: 14px; line-height: 1.2;",
            tabindex: 0,
            onkeydown: move |e| handle_key(e, &terminal),

            for row in 0..g.rows {
                div {
                    class: "terminal-row",
                    style: "display: flex; height: {cell_height}px;",

                    for col in 0..g.cols {
                        if let Some(cell) = g.cells.iter().find(|c| c.row == row as u16 && c.col == col as u16) {
                            {render_cell(cell, &color_scheme, cell_width)}
                        }
                    }
                }
            }

            // カーソル
            if g.cursor.visible {
                div {
                    class: "cursor",
                    style: "position: absolute; left: {g.cursor.col as f32 * cell_width}px; top: {g.cursor.row as f32 * cell_height}px; width: {cell_width}px; height: {cell_height}px; background: {color_scheme.cursor.to_css()}; opacity: 0.7;",
                }
            }
        }
    }
}

fn render_cell(cell: &CellInfo, scheme: &ColorScheme, width: f32) -> Element {
    let fg = scheme.resolve_color(cell.fg);
    let bg = scheme.resolve_color(cell.bg);

    let mut style = format!(
        "width: {}px; color: {}; background: {};",
        width,
        fg.to_css(),
        bg.to_css()
    );

    if cell.flags.contains(Flags::BOLD) {
        style.push_str(" font-weight: bold;");
    }
    if cell.flags.contains(Flags::ITALIC) {
        style.push_str(" font-style: italic;");
    }
    if cell.flags.contains(Flags::UNDERLINE) {
        style.push_str(" text-decoration: underline;");
    }

    rsx! {
        span {
            style: "{style}",
            "{cell.content}"
        }
    }
}

fn handle_key(e: KeyboardEvent, terminal: &Signal<TerminalManager>) {
    let key_bytes = match e.key() {
        Key::Character(c) => c.as_bytes().to_vec(),
        Key::Enter => vec![b'\r'],
        Key::Backspace => vec![0x7f],
        Key::Tab => vec![b'\t'],
        Key::Escape => vec![0x1b],
        Key::ArrowUp => vec![0x1b, b'[', b'A'],
        Key::ArrowDown => vec![0x1b, b'[', b'B'],
        Key::ArrowRight => vec![0x1b, b'[', b'C'],
        Key::ArrowLeft => vec![0x1b, b'[', b'D'],
        _ => return,
    };

    let mut term = terminal.write();
    let _ = term.write(&key_bytes);
}

impl Rgb {
    fn to_css(&self) -> String {
        format!("rgb({},{},{})", self.r, self.g, self.b)
    }
}
```

### 3.2 Canvas/WGPU描画（高パフォーマンス版）

```rust
// components/terminal/renderer.rs
// Phase 3.2で実装 - GPUアクセラレート描画

use dioxus::prelude::*;
use wgpu::*;

pub struct TerminalRenderer {
    device: Device,
    queue: Queue,
    surface: Surface,
    pipeline: RenderPipeline,
    glyph_cache: GlyphCache,
}

impl TerminalRenderer {
    /// グリッドをGPUで描画
    pub fn render(&mut self, grid: &TerminalGrid, scheme: &ColorScheme) {
        // 1. 背景色の矩形をバッチ描画
        // 2. グリフをアトラスから描画
        // 3. カーソルをオーバーレイ
        // 4. 選択範囲をハイライト
    }
}

pub struct GlyphCache {
    atlas: TextureAtlas,
    glyphs: HashMap<(char, Flags), GlyphInfo>,
}

impl GlyphCache {
    pub fn get_or_rasterize(&mut self, c: char, flags: Flags) -> &GlyphInfo {
        // fontdueでラスタライズしてアトラスに追加
    }
}
```

---

## Phase 4: Sphinxプレビュー

### 4.1 WebView埋め込み（wryを使用）

```rust
// components/preview.rs
use dioxus::prelude::*;

#[cfg(feature = "webview-preview")]
use wry::{WebView, WebViewBuilder};

#[component]
pub fn PreviewPane(sphinx_port: Option<u16>) -> Element {
    let webview = use_signal(|| None::<WebView>);

    use_effect(move || {
        if let Some(port) = sphinx_port {
            #[cfg(feature = "webview-preview")]
            {
                let url = format!("http://127.0.0.1:{}", port);
                // WebViewを初期化してsphinx-autobuildのページを表示
                let wv = WebViewBuilder::new()
                    .with_url(&url)
                    .build()
                    .expect("Failed to create webview");
                webview.set(Some(wv));
            }
        }
    });

    rsx! {
        div {
            class: "preview-container",
            style: "width: 100%; height: 100%;",
            id: "preview-webview",
            // WebViewがここにアタッチされる
        }
    }
}
```

### 4.2 代替: 外部ブラウザ連携

```rust
// WebView埋め込みが複雑な場合の代替
#[component]
pub fn PreviewPane(sphinx_port: Option<u16>) -> Element {
    rsx! {
        div {
            class: "preview-placeholder",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; background: #f5f5f5;",

            if let Some(port) = sphinx_port {
                p { "Sphinx server running on port {port}" }
                button {
                    onclick: move |_| {
                        let url = format!("http://127.0.0.1:{}", port);
                        let _ = open::that(&url);
                    },
                    "Open in Browser"
                }
            } else {
                p { "Sphinx server not running" }
            }
        }
    }
}
```

---

## Phase 5: 状態管理とフック

### 5.1 グローバル状態

```rust
// state/mod.rs
use dioxus::prelude::*;

#[derive(Clone)]
pub struct AppState {
    pub terminal: Signal<Option<TerminalManager>>,
    pub sphinx: Signal<SphinxState>,
    pub config: Signal<Config>,
}

#[derive(Clone, Default)]
pub struct SphinxState {
    pub port: Option<u16>,
    pub status: SphinxStatus,
    pub last_build: Option<String>,
}

#[derive(Clone, Default, PartialEq)]
pub enum SphinxStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Building,
    Error(String),
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}
```

### 5.2 カスタムフック

```rust
// state/terminal.rs
use dioxus::prelude::*;

pub fn use_terminal(session_id: &str) -> UseTerminal {
    let app = use_app_state();
    let grid = use_signal(|| None::<TerminalGrid>);

    // PTY出力の購読
    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(16));
            loop {
                interval.tick().await;
                if let Some(ref term) = *app.terminal.read() {
                    grid.set(Some(term.get_grid()));
                }
            }
        });
    });

    UseTerminal {
        grid,
        write: move |data: &[u8]| {
            if let Some(ref mut term) = *app.terminal.write() {
                let _ = term.write(data);
            }
        },
        resize: move |cols, rows| {
            if let Some(ref mut term) = *app.terminal.write() {
                term.resize(cols, rows);
            }
        },
    }
}

pub struct UseTerminal {
    pub grid: Signal<Option<TerminalGrid>>,
    pub write: impl Fn(&[u8]),
    pub resize: impl Fn(u16, u16),
}
```

---

## Phase 6: 機能完成

### 6.1 テキスト選択

```rust
// components/terminal/selection.rs
use dioxus::prelude::*;

#[derive(Clone, Default)]
pub struct Selection {
    pub start: Option<(u16, u16)>,
    pub end: Option<(u16, u16)>,
    pub active: bool,
}

impl Selection {
    pub fn contains(&self, row: u16, col: u16) -> bool {
        // 選択範囲内かどうかを判定
    }

    pub fn get_selected_text(&self, grid: &TerminalGrid) -> String {
        // 選択されたテキストを取得
    }
}
```

### 6.2 スクロールバック

```rust
// alacritty_terminalのスクロールバック機能を活用
impl TerminalManager {
    pub fn scroll(&mut self, delta: i32) {
        let mut term = self.term.lock();
        term.scroll_display(Scroll::Delta(delta));
    }

    pub fn scroll_to_bottom(&mut self) {
        let mut term = self.term.lock();
        term.scroll_display(Scroll::Bottom);
    }
}
```

### 6.3 リサイズ処理

```rust
// components/terminal/view.rs
#[component]
pub fn TerminalView(/* ... */) -> Element {
    let container_ref = use_signal(|| None::<web_sys::Element>);
    let size = use_signal(|| (80u16, 24u16));

    // ResizeObserver相当の処理
    use_effect(move || {
        spawn(async move {
            // コンテナサイズを監視
            // セルサイズで割って cols/rows を計算
            // terminal.resize(cols, rows) を呼び出し
        });
    });

    // 100msスロットリング
    let resize_debounced = use_debounce(Duration::from_millis(100), move |(cols, rows)| {
        if let Some(ref mut term) = *terminal.write() {
            term.resize(cols, rows);
        }
    });

    // ...
}
```

---

## Phase 7: ビルド・配布

### 7.1 Dioxus.toml

```toml
[application]
name = "Khafre"
default_platform = "native"

[native]
title = "Khafre - Sphinx Documentation Editor"
width = 1200
height = 800
resizable = true
decorations = true

[bundle]
identifier = "net.p3ac0ck.khafre"
icon = ["assets/icons/icon.png"]
```

### 7.2 CI/CD更新

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install Dioxus CLI
        run: cargo install dioxus-cli

      - name: Build
        run: dx build --release

      - name: Test
        run: cargo test

      - name: Clippy
        run: cargo clippy -- -D warnings
```

---

## 実装順序まとめ

| Phase | 内容 | 成果物 |
|-------|------|--------|
| 1 | 基盤構築 | 最小Dioxus Nativeアプリ + サービス層移行 |
| 2 | alacritty_terminal統合 | VTパーシング + グリッド管理 |
| 3 | ターミナル描画 | DOM描画 → Canvas/WGPU描画 |
| 4 | Sphinxプレビュー | WebView埋め込み or 外部ブラウザ |
| 5 | 状態管理 | Signals + カスタムフック |
| 6 | 機能完成 | 選択、スクロール、リサイズ |
| 7 | ビルド・配布 | CI/CD、バンドル設定 |

---

## リスクと対策

| リスク | 影響度 | 対策 |
|-------|-------|------|
| Dioxus Native不安定 | 高 | 0.6安定版使用、フォールバック用にDesktop版も検討 |
| alacritty_terminal API変更 | 中 | バージョン固定、ラッパー層で吸収 |
| WGPU描画の複雑さ | 高 | 初期はDOM描画、段階的にWGPU化 |
| WebViewプレビュー統合 | 中 | 代替として外部ブラウザ連携を用意 |
| パフォーマンス | 中 | ベンチマーク実施、ボトルネック特定 |

---

## 参考リンク

- [Dioxus Documentation](https://dioxuslabs.com/docs/0.6/)
- [alacritty_terminal crate](https://crates.io/crates/alacritty_terminal)
- [portable-pty crate](https://crates.io/crates/portable-pty)
- [egui_term (参考実装)](https://github.com/Harzu/egui_term)
- [Alacritty Source](https://github.com/alacritty/alacritty)
