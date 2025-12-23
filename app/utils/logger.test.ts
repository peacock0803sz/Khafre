import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { logger } from "./logger";

describe("logger", () => {
  const originalEnv = import.meta.env.DEV;

  beforeEach(() => {
    vi.spyOn(console, "log").mockImplementation(() => {});
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("in development mode", () => {
    beforeEach(() => {
      // @ts-expect-error DEV is readonly
      import.meta.env.DEV = true;
    });

    afterEach(() => {
      // @ts-expect-error DEV is readonly
      import.meta.env.DEV = originalEnv;
    });

    it("logger.log should call console.log", () => {
      logger.log("test message");
      expect(console.log).toHaveBeenCalledWith("test message");
    });

    it("logger.error should call console.error", () => {
      logger.error("error message");
      expect(console.error).toHaveBeenCalledWith("error message");
    });

    it("logger.warn should call console.warn", () => {
      logger.warn("warn message");
      expect(console.warn).toHaveBeenCalledWith("warn message");
    });

    it("should pass multiple arguments", () => {
      logger.log("message", { key: "value" }, 123);
      expect(console.log).toHaveBeenCalledWith("message", { key: "value" }, 123);
    });
  });

  describe("in production mode", () => {
    beforeEach(() => {
      // @ts-expect-error DEV is readonly
      import.meta.env.DEV = false;
    });

    afterEach(() => {
      // @ts-expect-error DEV is readonly
      import.meta.env.DEV = originalEnv;
    });

    it("logger.log should not call console.log", () => {
      logger.log("test message");
      expect(console.log).not.toHaveBeenCalled();
    });

    it("logger.error should not call console.error", () => {
      logger.error("error message");
      expect(console.error).not.toHaveBeenCalled();
    });

    it("logger.warn should not call console.warn", () => {
      logger.warn("warn message");
      expect(console.warn).not.toHaveBeenCalled();
    });
  });
});
