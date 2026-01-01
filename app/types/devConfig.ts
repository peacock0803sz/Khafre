import type { ProjectConfig, ColorScheme } from "./config";

/** 設定の部分上書き用型 */
export type ConfigOverride = {
  sphinx?: {
    source_dir?: string;
    build_dir?: string;
    server?: {
      port?: number;
    };
    extra_args?: string[];
  };
  python?: {
    interpreter?: string;
  };
  editor?: {
    command?: string;
  };
  terminal?: {
    shell?: string;
    font_family?: string;
    font_size?: number;
    theme_file?: string;
    color_scheme?: ColorScheme;
  };
};

/** ローカル開発用設定 (.khafre.dev.json) */
export interface DevConfig {
  /** デフォルトで開くプロジェクトパス */
  project_path?: string;
  /** sphinx-autobuildを自動起動するか */
  auto_start_sphinx?: boolean;
  /** グローバル設定の上書き */
  config?: ConfigOverride;
}

/** ConfigOverrideをProjectConfigにマージする */
export function mergeConfig(
  base: ProjectConfig,
  override: ConfigOverride | undefined
): ProjectConfig {
  if (!override) return base;

  return {
    sphinx: {
      source_dir: override.sphinx?.source_dir ?? base.sphinx.source_dir,
      build_dir: override.sphinx?.build_dir ?? base.sphinx.build_dir,
      server: {
        port: override.sphinx?.server?.port ?? base.sphinx.server.port,
      },
      extra_args: override.sphinx?.extra_args ?? base.sphinx.extra_args,
    },
    python: {
      interpreter: override.python?.interpreter ?? base.python.interpreter,
    },
    editor: {
      command: override.editor?.command ?? base.editor.command,
    },
    terminal: {
      shell: override.terminal?.shell ?? base.terminal.shell,
      font_family: override.terminal?.font_family ?? base.terminal.font_family,
      font_size: override.terminal?.font_size ?? base.terminal.font_size,
      theme_file: override.terminal?.theme_file ?? base.terminal.theme_file,
      color_scheme: override.terminal?.color_scheme ?? base.terminal.color_scheme,
    },
  };
}
