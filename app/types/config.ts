/** sphinx-autobuildサーバー設定 */
export interface ServerConfig {
  port: number;
}

/** Sphinx関連設定 */
export interface SphinxConfig {
  source_dir: string;
  build_dir: string;
  server: ServerConfig;
}

/** Python環境設定 */
export interface PythonConfig {
  interpreter: string;
}

/** エディタ設定 */
export interface EditorConfig {
  command: string;
}

/** プロジェクト設定全体 */
export interface ProjectConfig {
  sphinx: SphinxConfig;
  python: PythonConfig;
  editor: EditorConfig;
}
