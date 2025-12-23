import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface UseProjectDialogResult {
  projectPath: string | null;
  showDialog: () => Promise<string | null>;
  clearProject: () => void;
}

/**
 * プロジェクトフォルダ選択ダイアログを管理するhook
 */
export function useProjectDialog(): UseProjectDialogResult {
  const [projectPath, setProjectPath] = useState<string | null>(null);

  const showDialog = useCallback(async (): Promise<string | null> => {
    try {
      const selected = await open({
        title: "Select Sphinx Project Folder",
        directory: true,
        recursive: true,
      });

      if (selected && typeof selected === "string") {
        setProjectPath(selected);
        return selected;
      }

      return null;
    } catch (e) {
      console.error("Failed to open folder dialog:", e);
      return null;
    }
  }, []);

  const clearProject = useCallback(() => {
    setProjectPath(null);
  }, []);

  return { projectPath, showDialog, clearProject };
}
