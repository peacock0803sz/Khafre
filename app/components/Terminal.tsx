import { useEffect, useRef, useCallback } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
import "@xterm/xterm/css/xterm.css";

// デフォルトフォント設定
const DEFAULT_FONT_FAMILY = 'Menlo, Monaco, "Courier New", monospace';
const DEFAULT_FONT_SIZE = 14;

interface TerminalProps {
  sessionId: string;
  cwd?: string;
  shell?: string;
  fontFamily?: string;
  fontSize?: number;
  onExit?: (code: number) => void;
}

export function Terminal({ sessionId, cwd, shell, fontFamily, fontSize, onExit }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const resizeTimeoutRef = useRef<number | null>(null);

  // PTYにデータを送信
  const sendData = useCallback(
    async (data: string) => {
      try {
        await invoke("pty_write", { sessionId, data });
      } catch (e) {
        logger.error("Failed to write to PTY:", e);
      }
    },
    [sessionId]
  );

  // リサイズ（間引き処理付き）
  const handleResize = useCallback(() => {
    if (resizeTimeoutRef.current) {
      window.clearTimeout(resizeTimeoutRef.current);
    }

    // 100msの間引き（ドラッグ中の過剰なリサイズを防ぐ）
    resizeTimeoutRef.current = window.setTimeout(async () => {
      if (!terminalRef.current || !fitAddonRef.current) return;

      fitAddonRef.current.fit();
      const { cols, rows } = terminalRef.current;

      try {
        await invoke("pty_resize", { sessionId, cols, rows });
      } catch (e) {
        logger.error("Failed to resize PTY:", e);
      }
    }, 100);
  }, [sessionId]);

  useEffect(() => {
    if (!containerRef.current) return;

    // xterm.js初期化
    const terminal = new XTerm({
      cursorBlink: true,
      fontSize: fontSize ?? DEFAULT_FONT_SIZE,
      fontFamily: fontFamily ?? DEFAULT_FONT_FAMILY,
      scrollback: 10000,
      theme: {
        background: "#1e1e1e",
        foreground: "#d4d4d4",
        cursor: "#d4d4d4",
      },
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminal.open(containerRef.current);
    fitAddon.fit();

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // ユーザー入力をPTYに送信
    terminal.onData(sendData);

    // PTYセッション開始
    const { cols, rows } = terminal;
    invoke("spawn_terminal", { sessionId, cwd, shell, cols, rows }).catch((e) => {
      logger.error("Failed to spawn terminal:", e);
      terminal.write(`\r\nError: ${e}\r\n`);
    });

    // PTYからのデータを受信
    let unlistenData: UnlistenFn | null = null;
    let unlistenExit: UnlistenFn | null = null;

    const setupListeners = async () => {
      unlistenData = await listen<[string, string]>("pty_data", (event) => {
        const [sid, data] = event.payload;
        if (sid === sessionId) {
          terminal.write(data);
        }
      });

      unlistenExit = await listen<[string, number]>("pty_exit", (event) => {
        const [sid, code] = event.payload;
        if (sid === sessionId) {
          terminal.write(`\r\n[Process exited with code ${code}]\r\n`);
          onExit?.(code);
        }
      });
    };

    setupListeners();

    // リサイズ監視
    const resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(containerRef.current);

    // クリーンアップ
    return () => {
      if (resizeTimeoutRef.current) {
        window.clearTimeout(resizeTimeoutRef.current);
      }
      resizeObserver.disconnect();
      unlistenData?.();
      unlistenExit?.();
      terminal.dispose();

      // PTYセッション終了
      invoke("kill_terminal", { sessionId }).catch(logger.error);
    };
    // cwdは初回spawnのみ使用、変更時の再spawnは不要
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  return (
    <div ref={containerRef} className="w-full h-full" style={{ backgroundColor: "#1e1e1e" }} />
  );
}
