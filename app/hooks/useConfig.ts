import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "../types/config";

interface UseConfigResult {
  config: ProjectConfig | null;
  error: string | null;
  loading: boolean;
  reload: () => void;
}

/**
 * グローバル設定を読み込むhook
 * $XDG_CONFIG_HOME/khafre/config.toml から設定を読み込む
 */
export function useConfig(): UseConfigResult {
  const [config, setConfig] = useState<ProjectConfig | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const loadConfig = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const loadedConfig = await invoke<ProjectConfig>("load_config");
      setConfig(loadedConfig);
    } catch (e) {
      setError(String(e));
      setConfig(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  return { config, error, loading, reload: loadConfig };
}
