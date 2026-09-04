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

import { loadBootstrap, listWeeks, saveSettings, saveWeek, createOrSwitchWeek } from "$lib/api";
import { appStore } from "./app";

const mockedBootstrap = vi.mocked(loadBootstrap);
const mockedSaveSettings = vi.mocked(saveSettings);
const mockedSaveWeek = vi.mocked(saveWeek);
const mockedListWeeks = vi.mocked(listWeeks);
const mockedSwitch = vi.mocked(createOrSwitchWeek);

const ACTIVE_WEEK = {
  weekId: "w1",
  weekStart: "2026-08-31",
  entries: [],
  overtimeThresholdMinutes: 2100,
  travelDeductionMinutes: 30,
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
      activeWeekId: "w1",
      enableTravelDeduction: true,
      travelDeductionMinutes: 30,
      travelDeductionLabel: "30 min",
      vacationDayHours: 468,
      vacationDayHoursLabel: "7.8h"
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
      configuredDays: [],
      enableTravelDeduction: true,
      travelDeductionMinutes: 30,
      vacationDayHours: 468
    });

    // Sans le reload, la sidebar garderait 130 % / +10h30 (ancien seuil)
    // alors qu'un export dirait 128 % / +10h00 — divergence incontestable.
    expect(snapshot).toEqual({ percentage: 128, balance: "+10h00" });
    expect(mockedBootstrap).toHaveBeenCalledTimes(2);

    unsubscribe();
  });
});

describe("appStore.persistWeek", () => {
  const SAVE_INPUT = {
    weekId: "w1",
    weekStart: "2026-08-31",
    overtimeThresholdMinutes: 2100,
    travelDeductionMinutes: 30,
    entries: []
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockedBootstrap.mockResolvedValue({ activeWeek: ACTIVE_WEEK });
    mockedListWeeks.mockResolvedValue([]);
  });

  it("met à jour la semaine active depuis la vue retournée par save_week, sans recharger le bootstrap", async () => {
    const SAVED_WEEK = {
      ...ACTIVE_WEEK,
      summary: {
        totalLabel: "45h30",
        percentage: 128,
        cumulativeBalanceMinutes: 600,
        cumulativeBalanceLabel: "+10h00"
      }
    };
    mockedSaveWeek.mockResolvedValue(SAVED_WEEK);

    await appStore.bootstrap();
    await appStore.persistWeek(SAVE_INPUT);

    // save_week retourne déjà la vue avec les totaux recalculés :
    // un appel à load_bootstrap serait un IPC + un scan d'historique superflus
    expect(mockedBootstrap).toHaveBeenCalledTimes(1); // bootstrap() initial uniquement
    expect(mockedSaveWeek).toHaveBeenCalledTimes(1);

    let activePercentage: number | null = null;
    const unsubscribe = appStore.subscribe((state) => {
      if (state.activeWeek) {
        activePercentage = state.activeWeek.summary.percentage;
      }
    });
    expect(activePercentage).toBe(128);
    unsubscribe();
  });

  it("ne rafraîchit pas l'historique par défaut (scan complet évité)", async () => {
    mockedSaveWeek.mockResolvedValue(ACTIVE_WEEK);

    await appStore.bootstrap();
    mockedListWeeks.mockClear(); // bootstrap() charge déjà l'historique
    await appStore.persistWeek(SAVE_INPUT);

    expect(mockedListWeeks).not.toHaveBeenCalled();
  });

  it("rafraîchit l'historique quand demandé (onglet historique visible)", async () => {
    mockedSaveWeek.mockResolvedValue(ACTIVE_WEEK);

    await appStore.bootstrap();
    mockedListWeeks.mockClear(); // bootstrap() charge déjà l'historique
    await appStore.persistWeek(SAVE_INPUT, { refreshHistory: true });

    expect(mockedListWeeks).toHaveBeenCalledTimes(1);
  });

  it("une réponse de save tardive n'écrase pas la semaine affichée après un switch", async () => {
    let resolveSave: (week: typeof ACTIVE_WEEK) => void;
    mockedSaveWeek.mockImplementation(
      () => new Promise((resolve) => (resolveSave = resolve))
    );
    const OTHER_WEEK = { ...ACTIVE_WEEK, weekId: "w2", weekStart: "2026-09-07" };
    mockedSwitch.mockResolvedValue(OTHER_WEEK);

    await appStore.bootstrap();
    const saving = appStore.persistWeek(SAVE_INPUT); // save de w1 en vol
    await appStore.switchWeek("2026-09-07"); // l'utilisateur change de semaine

    resolveSave!(ACTIVE_WEEK); // la réponse de w1 arrive après le switch
    await saving;

    let activeWeekId: string | null = null;
    const unsubscribe = appStore.subscribe((state) => {
      if (state.activeWeek) activeWeekId = state.activeWeek.weekId;
    });
    expect(activeWeekId).toBe("w2");
    unsubscribe();
  });
});

describe("appStore.refreshHistory", () => {
  it("recharge l'historique sans toucher au reste (ouverture de l'onglet)", async () => {
    vi.clearAllMocks();
    mockedBootstrap.mockResolvedValue({ activeWeek: ACTIVE_WEEK });
    const FRESH = [{ weekId: "w1", weekStart: "2026-08-31" }] as never;
    mockedListWeeks.mockResolvedValue(FRESH);

    await appStore.bootstrap();
    mockedListWeeks.mockClear();

    let history: unknown = null;
    const unsubscribe = appStore.subscribe((state) => (history = state.history));

    await appStore.refreshHistory();

    expect(mockedListWeeks).toHaveBeenCalledTimes(1);
    expect(history).toEqual(FRESH);

    unsubscribe();
  });
});
