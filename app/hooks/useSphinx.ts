import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { ProjectConfig } from "../types/config";

interface UseSphinxOptions {
  sessionId: string;
  projectPath: string | null;
  config: ProjectConfig | null;
}

interface UseSphinxResult {
  previewUrl: string | null;
  isRunning: boolean;
  error: string | null;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

/**
 * sphinx-autobuildプロセスを管理するhook
 */
export function useSphinx({ sessionId, projectPath, config }: UseSphinxOptions): UseSphinxResult {
  const [port, setPort] = useState<number | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const previewUrl = port ? `http://127.0.0.1:${port}` : null;

  const start = useCallback(async () => {
    if (!projectPath || !config) {
      setError("Project path or config is missing");
      return;
    }

    try {
      setError(null);
      const assignedPort = await invoke<number>("start_sphinx", {
        sessionId,
        projectPath,
        sourceDir: config.sphinx.source_dir,
        buildDir: config.sphinx.build_dir,
        pythonPath: config.python.interpreter,
        port: config.sphinx.server.port,
      });
      setPort(assignedPort);
      setIsRunning(true);
    } catch (e) {
      setError(String(e));
      setIsRunning(false);
    }
  }, [sessionId, projectPath, config]);

  const stop = useCallback(async () => {
    try {
      await invoke("stop_sphinx", { sessionId });
      setPort(null);
      setIsRunning(false);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [sessionId]);

  // Sphinxイベントをリッスン
  useEffect(() => {
    let unlistenStarted: UnlistenFn | null = null;
    let unlistenError: UnlistenFn | null = null;
    let unlistenBuilt: UnlistenFn | null = null;

    const setup = async () => {
      unlistenStarted = await listen<[string, number]>("sphinx_started", (event) => {
        const [sid, assignedPort] = event.payload;
        if (sid === sessionId) {
          setPort(assignedPort);
          setIsRunning(true);
        }
      });

      unlistenError = await listen<[string, string]>("sphinx_error", (event) => {
        const [sid, errorMsg] = event.payload;
        if (sid === sessionId) {
          setError(errorMsg);
        }
      });

      unlistenBuilt = await listen<string>("sphinx_built", (event) => {
        if (event.payload === sessionId) {
          // ビルド完了時にエラーをクリア
          setError(null);
        }
      });
    };

    setup();

    return () => {
      unlistenStarted?.();
      unlistenError?.();
      unlistenBuilt?.();
    };
  }, [sessionId]);

  // アンマウント時にSphinxを停止
  useEffect(() => {
    return () => {
      invoke("stop_sphinx", { sessionId }).catch(() => {
        // 停止エラーは無視
      });
    };
  }, [sessionId]);

  return { previewUrl, isRunning, error, start, stop };
}
