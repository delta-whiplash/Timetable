import { describe, expect, it } from "vitest";
import { initialAppState, toCommandError } from "./state";

describe("state helpers", () => {
  it("creates a deterministic initial UI state", () => {
    const state = initialAppState();

    expect(state.bootstrapped).toBe(false);
    expect(state.loading).toBe(true);
    expect(state.activeWeek).toBeNull();
    expect(state.history).toEqual([]);
    expect(state.notice).toBeNull();
  });

  it("maps invoke failures to a safe command error", () => {
    const error = toCommandError({ message: "backend unavailable" });

    expect(error.code).toBe("frontend.invoke_failed");
    expect(error.message).toBe("backend unavailable");
    expect(error.correlationId).toHaveLength(36);
  });
});
