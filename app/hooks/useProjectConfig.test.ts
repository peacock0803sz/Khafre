import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useProjectConfig } from "./useProjectConfig";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockConfig = {
  sphinx: {
    source_dir: "docs",
    build_dir: "_build/html",
    server: { port: 0 },
  },
  python: { interpreter: "python" },
  editor: { command: "nvim" },
};

describe("useProjectConfig", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return null config when projectPath is null", () => {
    const { result } = renderHook(() => useProjectConfig(null));

    expect(result.current.config).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("should load config when projectPath is provided", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useProjectConfig("/path/to/project"));

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.config).toEqual(mockConfig);
    expect(result.current.error).toBeNull();
    expect(invoke).toHaveBeenCalledWith("load_project_config", {
      path: "/path/to/project",
    });
  });

  it("should handle error when loading config fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Config not found"));

    const { result } = renderHook(() => useProjectConfig("/invalid/path"));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.config).toBeNull();
    expect(result.current.error).toBe("Error: Config not found");
  });

  it("should reload config when projectPath changes", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result, rerender } = renderHook(({ path }) => useProjectConfig(path), {
      initialProps: { path: "/first/path" },
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledWith("load_project_config", {
      path: "/first/path",
    });

    rerender({ path: "/second/path" });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("load_project_config", {
        path: "/second/path",
      });
    });
  });

  it("should reload config when reload() is called", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useProjectConfig("/path/to/project"));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledTimes(1);

    act(() => {
      result.current.reload();
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledTimes(2);
    });
  });

  it("should clear config when projectPath becomes null", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result, rerender } = renderHook(({ path }) => useProjectConfig(path), {
      initialProps: { path: "/path/to/project" as string | null },
    });

    await waitFor(() => {
      expect(result.current.config).toEqual(mockConfig);
    });

    rerender({ path: null });

    expect(result.current.config).toBeNull();
  });
});
