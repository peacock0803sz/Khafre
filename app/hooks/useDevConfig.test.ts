import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useDevConfig } from "./useDevConfig";

// @tauri-apps/api/core をモック
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("useDevConfig", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return null initially and loaded=false", async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    const { result } = renderHook(() => useDevConfig());

    // 初期状態を確認
    expect(result.current.devConfig).toBeNull();
    expect(result.current.loaded).toBe(false);

    // 非同期処理の完了を待つ
    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });
  });

  it("should load dev config successfully", async () => {
    const mockConfig = {
      project_path: "/path/to/project",
      auto_start_sphinx: true,
    };
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useDevConfig());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });

    expect(result.current.devConfig).toEqual(mockConfig);
    expect(invoke).toHaveBeenCalledWith("load_dev_config");
  });

  it("should handle null config (no .khafre.dev.json)", async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    const { result } = renderHook(() => useDevConfig());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });

    expect(result.current.devConfig).toBeNull();
  });

  it("should handle error gracefully", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Failed to load"));

    const { result } = renderHook(() => useDevConfig());

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });

    expect(result.current.devConfig).toBeNull();
  });
});
