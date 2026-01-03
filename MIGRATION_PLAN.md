# Tauri → Dioxus 移行計画

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
| コンポーネント | 現在 | ファイル数 |
|---------------|------|-----------|
| UI Framework | React 19 + TypeScript | 19 |
| スタイリング | Tailwind CSS | - |
| ターミナル | xterm.js | 1 |
| IPC | Tauri Commands | 10コマンド |
| PTY管理 | portable-pty | 1 |
| Sphinx管理 | tokio + subprocess | 1 |
| 設定 | TOML + JSON | 2 |

---

## 移行オプション

### Option A: Dioxus Desktop (WebView) - 推奨
```
┌─────────────────────────────────────────────────────────┐
│                   Dioxus Desktop                        │
├─────────────────────────────────────────────────────────┤
│             Dioxus (Rust + RSX)                         │
│   ├── UI Components (Rust)                              │
│   ├── State Management (Signals)                        │
│   ├── PTY/Sphinx (直接呼び出し)                          │
│   └── xterm.js (JavaScript Interop)                     │
├─────────────────────────────────────────────────────────┤
│                   WebView (Wry)                         │
└─────────────────────────────────────────────────────────┘
```

**メリット:**
- xterm.jsをそのまま利用可能（JavaScript interop経由）
- 移行が段階的に可能
- CSS/TailwindをほぼそのままmandalioCSSとして活用
- Dioxus DesktopはWryを使用（Tauriと同じWebView基盤）

**デメリット:**
- 依然としてWebView依存
- JavaScript interopの複雑さ

### Option B: Dioxus Native + alacritty-terminal
```
┌─────────────────────────────────────────────────────────┐
│                   Dioxus Native                         │
├─────────────────────────────────────────────────────────┤
│   Dioxus Blitz/Native (Rust)                            │
│   ├── Native UI (GPU rendered)                          │
│   ├── alacritty-terminal (VT処理)                       │
│   └── Custom terminal renderer                          │
└─────────────────────────────────────────────────────────┘
```

**メリット:**
- 完全ネイティブ、高パフォーマンス
- 依存関係が少ない

**デメリット:**
- ターミナルレンダリングを自作する必要
- 移行コストが非常に高い
- Dioxus Nativeはまだ実験的

### Option C: ハイブリッド（段階的移行）
1. Phase 1: バックエンドをDioxus対応にリファクタ
2. Phase 2: UIをDioxusに段階的に移行（xterm.jsはiframe経由で維持）
3. Phase 3: 将来的にネイティブターミナルに置き換え

---

## 推奨: Option A (Dioxus Desktop)

### 理由
1. **xterm.jsの継続利用**: 既存のターミナル機能を維持
2. **移行リスクが低い**: WebView基盤が同じWry
3. **開発速度**: 段階的な移行が可能
4. **成熟度**: Dioxus Desktopは最も安定

---

## 移行計画（Phase別）

### Phase 1: 準備とインフラ整備 (Week 1)

#### 1.1 プロジェクト構造の変更
```
Khafre/
├── src/                    # Dioxus アプリケーション
│   ├── main.rs
│   ├── app.rs              # メインApp Component
│   ├── components/         # Dioxusコンポーネント
│   │   ├── mod.rs
│   │   ├── terminal.rs
│   │   ├── preview.rs
│   │   └── layout/
│   │       ├── mod.rs
│   │       ├── split_view.rs
│   │       └── pane.rs
│   ├── hooks/              # Dioxus Hooks
│   │   ├── mod.rs
│   │   ├── use_config.rs
│   │   ├── use_sphinx.rs
│   │   └── use_pty.rs
│   ├── services/           # ビジネスロジック（backから移行）
│   │   ├── mod.rs
│   │   ├── terminal.rs     # PTY管理
│   │   ├── sphinx.rs       # Sphinx管理
│   │   ├── config.rs       # 設定管理
│   │   └── color_scheme.rs
│   └── types/              # 型定義
│       ├── mod.rs
│       └── config.rs
├── assets/                 # 静的アセット
│   ├── terminal.js         # xterm.js wrapper
│   └── styles.css          # Tailwind compiled
├── Cargo.toml
├── Dioxus.toml             # Dioxus設定
└── tailwind.config.js
```

#### 1.2 依存関係
```toml
# Cargo.toml
[dependencies]
dioxus = { version = "0.6", features = ["desktop"] }
dioxus-desktop = "0.6"

# 既存のバックエンド依存（維持）
portable-pty = "0.9"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.9"
dirs = "6"
rand = "0.9"

# 追加
wry = "0.50"  # JavaScript interop用
```

### Phase 2: コア機能の移行 (Week 2-3)

#### 2.1 状態管理の移行
```rust
// React hooks → Dioxus Signals
// useConfig.ts → use_config.rs
use dioxus::prelude::*;

#[derive(Clone, Default)]
pub struct AppState {
    pub config: Signal<Option<Config>>,
    pub sphinx_port: Signal<Option<u16>>,
    pub terminal_ready: Signal<bool>,
}

pub fn use_config() -> Signal<Option<Config>> {
    let config = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            match load_config().await {
                Ok(c) => config.set(Some(c)),
                Err(e) => log::error!("Config load failed: {}", e),
            }
        });
    });

    config
}
```

#### 2.2 コンポーネント移行マッピング
| React Component | Dioxus Component | 備考 |
|----------------|------------------|------|
| App.tsx | app.rs | メインレイアウト |
| Terminal.tsx | terminal.rs | JS interop必要 |
| Preview.tsx | preview.rs | iframe利用 |
| SplitView.tsx | split_view.rs | ドラッグ対応 |
| Pane.tsx | pane.rs | シンプル |

#### 2.3 ターミナル（JavaScript Interop）
```rust
// src/components/terminal.rs
use dioxus::prelude::*;
use dioxus_desktop::use_eval;

#[component]
pub fn Terminal(session_id: String) -> Element {
    let eval = use_eval();

    // xterm.js初期化
    use_effect(move || {
        eval(r#"
            const term = new Terminal({
                scrollback: 10000,
                fontFamily: 'Menlo, Monaco, monospace',
                fontSize: 14,
            });
            term.open(document.getElementById('terminal-container'));
            // ... 残りの初期化
        "#);
    });

    rsx! {
        div {
            id: "terminal-container",
            class: "w-full h-full"
        }
    }
}
```

### Phase 3: バックエンド統合 (Week 3-4)

#### 3.1 IPCの廃止
Tauri CommandsをRust内の直接呼び出しに置き換え:

```rust
// Before (Tauri IPC)
// Frontend: invoke('spawn_terminal', { session_id, ... })
// Backend: #[tauri::command] fn spawn_terminal(...)

// After (Direct call)
// services/terminal.rs
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl TerminalManager {
    pub async fn spawn(&self, session_id: &str, cwd: &str) -> Result<(), Error> {
        // 直接PTYを生成
    }
}
```

#### 3.2 イベントシステム
```rust
// Tauri events → Dioxus Signals/Coroutines
use dioxus::prelude::*;

// グローバルイベントチャンネル
static PTY_OUTPUT: GlobalSignal<Option<(String, Vec<u8>)>> = Signal::global(|| None);

// PTY読み取りタスク
spawn(async move {
    loop {
        if let Some(data) = pty_reader.read().await {
            PTY_OUTPUT.set(Some((session_id.clone(), data)));
        }
    }
});
```

### Phase 4: UI/UXの完成 (Week 4-5)

#### 4.1 スタイリング
- Tailwind CSS → DioxusのインラインスタイルまたはManganis
- カラースキーム対応の維持

#### 4.2 機能テスト
- PTY動作確認
- Sphinxプレビュー確認
- リサイズ動作確認
- テーマ切り替え確認

### Phase 5: クリーンアップ (Week 5)

- 不要なファイル削除（app/, back/）
- ドキュメント更新
- CI/CD更新

---

## アーキテクチャのリファクタリング

### 現在の問題点
1. **二言語分離**: TypeScript + Rustで重複ロジック
2. **IPC オーバーヘッド**: JSON シリアライズ/デシリアライズ
3. **複雑なビルドパイプライン**: Vite + Tauri

### Dioxus移行後の改善
1. **単一言語**: Rust のみ
2. **直接呼び出し**: IPC 不要
3. **シンプルなビルド**: `dx build` のみ

### 新アーキテクチャ
```
┌─────────────────────────────────────────────────────────┐
│                   Khafre (Dioxus)                       │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌────────────────────────────┐  │
│  │   UI Layer       │  │    Service Layer           │  │
│  │  (Components)    │──│  (Terminal, Sphinx, Config)│  │
│  │                  │  │                            │  │
│  │  - Terminal      │  │  - TerminalManager         │  │
│  │  - Preview       │  │  - SphinxManager           │  │
│  │  - SplitView     │  │  - ConfigManager           │  │
│  └──────────────────┘  └────────────────────────────┘  │
│           │                        │                    │
│           ▼                        ▼                    │
│  ┌──────────────────┐  ┌────────────────────────────┐  │
│  │  State Layer     │  │    External Layer          │  │
│  │  (Signals)       │  │  (portable-pty, subprocess)│  │
│  └──────────────────┘  └────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│            WebView (Wry) + JS Interop (xterm.js)        │
└─────────────────────────────────────────────────────────┘
```

---

## リスクと対策

| リスク | 影響度 | 対策 |
|-------|-------|------|
| xterm.js interopの複雑さ | 高 | プロトタイプで早期検証 |
| Dioxus APIの変更 | 中 | 0.6 LTS使用、依存固定 |
| パフォーマンス劣化 | 中 | ベンチマーク比較 |
| 機能欠落 | 中 | 移行前に機能リスト作成 |

---

## 次のステップ

1. **技術検証（PoC）**
   - [ ] Dioxus Desktop + xterm.js の動作確認
   - [ ] PTY出力のJS bridging検証
   - [ ] パフォーマンス計測

2. **承認後の実装開始**
   - Phase 1から順次実行

---

## 質問事項

移行を進める前に確認したい点:

1. **レンダリング戦略**: Option A (Desktop/WebView) で進めてよいか？
2. **ターミナル**: xterm.js継続 vs 将来的にRustネイティブ？
3. **移行スコープ**: 全機能移行 vs MVPから開始？
4. **スケジュール優先度**: 品質 vs スピード？
