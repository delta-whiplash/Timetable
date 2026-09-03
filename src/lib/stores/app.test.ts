import { describe, expect, it, vi, beforeEach } from "vitest";

// Mock de l'API Tauri avant l'import du store
vi.mock("$lib/api", () => ({
  loadBootstrap: vi.fn(),
  saveWeek: vi.fn(),
  createOrSwitchWeek: vi.fn(),
  listWeeks: vi.fn(),
  deleteWeek: vi.fn(),
  loadSettings: vi.fn(),
  saveSettings: vi.fn(),
  setTheme: vi.fn(),
  exportWeek: vi.fn(),
  getAnalytics: vi.fn()
}));

import { loadBootstrap, saveSettings } from "$lib/api";
import { appStore } from "./app";

const mockedBootstrap = vi.mocked(loadBootstrap);
const mockedSaveSettings = vi.mocked(saveSettings);

const ACTIVE_WEEK = {
  weekId: "w1",
  weekStart: "2026-08-31",
  entries: [],
  overtimeThresholdMinutes: 2100,
  summary: {
    totalLabel: "45h30",
    percentage: 130,
    cumulativeBalanceMinutes: 630,
    cumulativeBalanceLabel: "+10h30"
  }
};

describe("appStore.persistSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Bootstrap initial : semaine avec l'ancien seuil (35h)
    mockedBootstrap.mockResolvedValue({ activeWeek: ACTIVE_WEEK });
    mockedSaveSettings.mockResolvedValue({
      overtimeThresholdMinutes: 2130,
      overtimeThresholdLabel: "35h 30min",
      theme: "dark",
      defaultStart: "08:00",
      defaultEnd: "18:00",
      defaultBreak: "01:00",
      configuredDays: [],
      activeWeekId: "w1"
    });
  });

  it("recharge la semaine active après enregistrement des paramètres", async () => {
    // Le backend recalcule le seuil de la semaine courante : la vue rechargée
    // diffère de celle affichée avant le save (128 %, +10h00 au lieu de 130 %, +10h30)
    const RELOADED_WEEK = {
      ...ACTIVE_WEEK,
      overtimeThresholdMinutes: 2130,
      summary: {
        totalLabel: "45h30",
        percentage: 128,
        cumulativeBalanceMinutes: 600,
        cumulativeBalanceLabel: "+10h00"
      }
    };
    mockedBootstrap
      .mockResolvedValueOnce({ activeWeek: ACTIVE_WEEK }) // bootstrap() initial
      .mockResolvedValueOnce({ activeWeek: RELOADED_WEEK }); // reload après persistSettings

    await appStore.bootstrap();

    let snapshot: { percentage: number; balance: string } | null = null;
    const unsubscribe = appStore.subscribe((state) => {
      if (state.activeWeek) {
        snapshot = {
          percentage: state.activeWeek.summary.percentage,
          balance: state.activeWeek.summary.cumulativeBalanceLabel
        };
      }
    });

    await appStore.persistSettings({
      overtimeThresholdMinutes: 2130,
      defaultStart: "08:00",
      defaultEnd: "18:00",
      defaultBreak: "01:00",
      configuredDays: []
    });

    // Sans le reload, la sidebar garderait 130 % / +10h30 (ancien seuil)
    // alors qu'un export dirait 128 % / +10h00 — divergence incontestable.
    expect(snapshot).toEqual({ percentage: 128, balance: "+10h00" });
    expect(mockedBootstrap).toHaveBeenCalledTimes(2);

    unsubscribe();
  });
});
