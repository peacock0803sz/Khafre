import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "../types/config";

interface UseProjectConfigResult {
  config: ProjectConfig | null;
  error: string | null;
  loading: boolean;
  reload: () => void;
}

/**
 * プロジェクト設定を読み込むhook
 * @param projectPath プロジェクトのパス（nullの場合は読み込まない）
 */
export function useProjectConfig(projectPath: string | null): UseProjectConfigResult {
  const [config, setConfig] = useState<ProjectConfig | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const loadConfig = async (path: string) => {
    setLoading(true);
    setError(null);

    try {
      const loadedConfig = await invoke<ProjectConfig>("load_project_config", { path });
      setConfig(loadedConfig);
    } catch (e) {
      setError(String(e));
      setConfig(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!projectPath) {
      setConfig(null);
      return;
    }

    loadConfig(projectPath);
  }, [projectPath]);

  const reload = () => {
    if (projectPath) {
      loadConfig(projectPath);
    }
  };

  return { config, error, loading, reload };
}
