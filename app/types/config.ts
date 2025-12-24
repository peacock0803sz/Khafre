/** sphinx-autobuildサーバー設定 */
export interface ServerConfig {
  port: number;
}

/** Sphinx関連設定 */
export interface SphinxConfig {
  source_dir: string;
  build_dir: string;
  server: ServerConfig;
  extra_args: string[];
}

/** Python環境設定 */
export interface PythonConfig {
  interpreter: string;
}

/** エディタ設定 */
export interface EditorConfig {
  command: string;
}

/** ターミナル設定 */
export interface TerminalConfig {
  shell?: string;
  font_family?: string;
  font_size?: number;
}

/** プロジェクト設定全体 */
export interface ProjectConfig {
  sphinx: SphinxConfig;
  python: PythonConfig;
  editor: EditorConfig;
  terminal: TerminalConfig;
}
