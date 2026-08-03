import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("onboarding IPC", () => {
  beforeEach(() => invoke.mockReset());

  it("exposes native onboarding preview only on macOS", async () => {
    const api = await import("./api");
    expect(api.supportsOnboardingPreview("macos")).toBe(true);
    expect(api.supportsOnboardingPreview("windows")).toBe(false);
  });

  it("uses stable commands and the onboarding argument name", async () => {
    invoke.mockResolvedValue({});
    const api = await import("./api");
    const state = {
      completed_revision: null,
      current_step: "practice" as const,
      visited_steps: ["overview" as const, "practice" as const],
      selected_example: "text" as const,
    };
    await api.saveOnboardingState(state);
    expect(invoke).toHaveBeenCalledWith("save_onboarding_state", { onboarding: state });
    await api.getAutoPasteReadiness();
    expect(invoke).toHaveBeenLastCalledWith("get_auto_paste_readiness");
  });
});
