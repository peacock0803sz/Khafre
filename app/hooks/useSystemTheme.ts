import { useState, useEffect } from "react";

export type SystemTheme = "light" | "dark";

/**
 * OSのLight/Darkテーマ設定を検出・監視するフック
 * テーマが変更されるとリアルタイムで更新される
 */
export function useSystemTheme(): SystemTheme {
  const [theme, setTheme] = useState<SystemTheme>(() => {
    // SSR対応: windowがない場合はdarkをデフォルトに
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  });

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      setTheme(e.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, []);

  return theme;
}
