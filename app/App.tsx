import { useState, useCallback, useEffect, useMemo } from "react";
import { Terminal } from "./components/Terminal";
import { Preview } from "./components/Preview";
import { SplitView, Pane } from "./components/layout";
import { useProjectDialog } from "./hooks/useProjectDialog";
import { useConfig } from "./hooks/useConfig";
import { useSphinx } from "./hooks/useSphinx";
import { useDevConfig } from "./hooks/useDevConfig";
import { mergeConfig } from "./types/devConfig";
import "./App.css";

function App() {
  const [exited, setExited] = useState(false);

  // ローカル開発用設定
  const { devConfig, loaded: devConfigLoaded } = useDevConfig();

  // プロジェクト選択
  const { projectPath, setProjectPath, showDialog } = useProjectDialog();

  // dev configからプロジェクトパスを設定
  useEffect(() => {
    if (devConfigLoaded && devConfig?.project_path && !projectPath) {
      setProjectPath(devConfig.project_path);
    }
  }, [devConfigLoaded, devConfig, projectPath, setProjectPath]);

  // projectPathが変わったら新しいsessionIdを生成（ターミナル再起動）
  const [sessionId, setSessionId] = useState(() => crypto.randomUUID());
  useEffect(() => {
    if (projectPath) {
      setSessionId(crypto.randomUUID());
      setExited(false);
    }
  }, [projectPath]);
  const { config, loading: configLoading } = useConfig();

  // devConfigによる設定の上書きをマージ
  const effectiveConfig = useMemo(() => {
    if (!config) return null;
    return mergeConfig(config, devConfig?.config);
  }, [config, devConfig?.config]);

  // sphinx-autobuild
  const {
    previewUrl,
    isRunning: sphinxRunning,
    error: sphinxError,
    start: startSphinx,
    stop: stopSphinx,
    openInBrowser,
  } = useSphinx({ sessionId, projectPath, config: effectiveConfig });

  const handleExit = useCallback((_code: number) => {
    setExited(true);
  }, []);

  // 起動時にプロジェクト選択ダイアログを表示（dev configが無い場合のみ）
  useEffect(() => {
    if (devConfigLoaded && !projectPath && !devConfig?.project_path) {
      showDialog();
    }
    // showDialogは安定した参照なので依存配列から除外
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [devConfigLoaded, projectPath, devConfig]);

  // config読み込み完了時にsphinx-autobuildを自動起動
  const autoStartSphinx = devConfig?.auto_start_sphinx ?? true;
  useEffect(() => {
    if (effectiveConfig && projectPath && !sphinxRunning && autoStartSphinx) {
      startSphinx();
    }
    // 初回起動時のみ実行、sphinxRunning/startSphinxの変更では再実行しない
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [effectiveConfig, projectPath, autoStartSphinx]);

  return (
    <main className="h-screen w-screen flex flex-col bg-gray-900">
      <header className="h-8 bg-gray-800 flex items-center justify-between px-4 text-gray-300 text-sm shrink-0">
        <span className="flex items-center gap-2">
          Orthrus
          {projectPath && (
            <span className="text-gray-500 text-xs truncate max-w-md">{projectPath}</span>
          )}
        </span>
        <div className="flex items-center gap-4">
          {configLoading && <span className="text-yellow-400 text-xs">Loading...</span>}
          {sphinxRunning && !previewUrl && (
            <span className="text-yellow-400 text-xs">Building...</span>
          )}
          {sphinxRunning && previewUrl && (
            <span className="text-green-400 text-xs">Preview Running</span>
          )}
          {sphinxError && (
            <span className="text-red-400 text-xs truncate max-w-xs">{sphinxError}</span>
          )}
          {sphinxRunning ? (
            <>
              <button
                onClick={openInBrowser}
                className="px-2 py-0.5 bg-blue-700 hover:bg-blue-600 rounded text-xs transition-colors"
              >
                Open in Browser
              </button>
              <button
                onClick={stopSphinx}
                className="px-2 py-0.5 bg-red-700 hover:bg-red-600 rounded text-xs transition-colors"
              >
                Stop Preview
              </button>
            </>
          ) : (
            effectiveConfig && (
              <button
                onClick={startSphinx}
                className="px-2 py-0.5 bg-green-700 hover:bg-green-600 rounded text-xs transition-colors"
              >
                Start Preview
              </button>
            )
          )}
          <button
            onClick={showDialog}
            className="px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-xs transition-colors"
          >
            Open Project
          </button>
        </div>
      </header>
      <div className="flex-1 min-h-0">
        <SplitView
          left={
            <Pane>
              <Preview url={previewUrl} isBuilding={sphinxRunning && !previewUrl} />
            </Pane>
          }
          right={
            <Pane>
              {projectPath && effectiveConfig && !exited ? (
                <Terminal
                  sessionId={sessionId}
                  cwd={projectPath}
                  shell={effectiveConfig.terminal.shell}
                  fontFamily={effectiveConfig.terminal.font_family}
                  fontSize={effectiveConfig.terminal.font_size}
                  colorScheme={effectiveConfig.terminal.color_scheme}
                  onExit={handleExit}
                />
              ) : (
                <div className="flex items-center justify-center h-full text-gray-400">
                  {exited ? "Terminal session ended" : "Select a project to start terminal"}
                </div>
              )}
            </Pane>
          }
        />
      </div>
    </main>
  );
}

export default App;
