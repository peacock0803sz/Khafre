import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DevConfig } from "../types/devConfig";

/**
 * ローカル開発用設定を読み込むhook
 * .khafre.dev.json が存在する場合のみ設定を返す
 */
export function useDevConfig() {
  const [devConfig, setDevConfig] = useState<DevConfig | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    invoke<DevConfig | null>("load_dev_config")
      .then((config) => {
        setDevConfig(config);
        setLoaded(true);
      })
      .catch(() => {
        setLoaded(true);
      });
  }, []);

  return { devConfig, loaded };
}
