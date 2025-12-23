import { useState, useCallback, useEffect } from "react";
import { Terminal } from "./components/Terminal";
import { Preview } from "./components/Preview";
import { SplitView, Pane } from "./components/layout";
import { useProjectDialog } from "./hooks/useProjectDialog";
import { useProjectConfig } from "./hooks/useProjectConfig";
import { useSphinx } from "./hooks/useSphinx";
import { useDevConfig } from "./hooks/useDevConfig";
import "./App.css";

function App() {
  const [exited, setExited] = useState(false);

  // ローカル開発用設定
  const { devConfig, loaded: devConfigLoaded } = useDevConfig();

  // プロジェクト選択
  const { projectPath, setProjectPath, showDialog } = useProjectDialog();

  // dev configからプロジェクトパスを設定
  useEffect(() => {
    if (devConfigLoaded && devConfig?.projectPath && !projectPath) {
      setProjectPath(devConfig.projectPath);
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
  const { config, loading: configLoading } = useProjectConfig(projectPath);

  // sphinx-autobuild
  const {
    previewUrl,
    isRunning: sphinxRunning,
    error: sphinxError,
    start: startSphinx,
    stop: stopSphinx,
  } = useSphinx({ sessionId, projectPath, config });

  const handleExit = useCallback((code: number) => {
    console.log("Terminal exited with code:", code);
    setExited(true);
  }, []);

  // 起動時にプロジェクト選択ダイアログを表示（dev configが無い場合のみ）
  useEffect(() => {
    if (devConfigLoaded && !projectPath && !devConfig?.projectPath) {
      showDialog();
    }
  }, [devConfigLoaded, projectPath, devConfig]);

  // config読み込み完了時にsphinx-autobuildを自動起動
  const autoStartSphinx = devConfig?.autoStartSphinx ?? true;
  useEffect(() => {
    if (config && projectPath && !sphinxRunning && autoStartSphinx) {
      startSphinx();
    }
  }, [config, projectPath, autoStartSphinx]);

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
          {sphinxRunning && <span className="text-green-400 text-xs">Preview Running</span>}
          {sphinxError && (
            <span className="text-red-400 text-xs truncate max-w-xs">{sphinxError}</span>
          )}
          {sphinxRunning ? (
            <button
              onClick={stopSphinx}
              className="px-2 py-0.5 bg-red-700 hover:bg-red-600 rounded text-xs transition-colors"
            >
              Stop Preview
            </button>
          ) : (
            config && (
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
              <Preview url={previewUrl} />
            </Pane>
          }
          right={
            <Pane>
              {projectPath && !exited ? (
                <Terminal sessionId={sessionId} cwd={projectPath} onExit={handleExit} />
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
