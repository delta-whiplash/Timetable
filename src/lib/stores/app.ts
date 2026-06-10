import { writable } from "svelte/store";
import type {
  AnalyticsDataView,
  BootstrapState,
  SaveSettingsInput,
  SaveWeekInput,
  ThemePreference,
  WeekListItem,
  WeekSheetView
} from "$lib/types";
import {
  createOrSwitchWeek,
  deleteWeek,
  getAnalytics,
  getAppStatus,
  listWeeks,
  loadBootstrap,
  loadSettings,
  saveSettings,
  saveWeek,
  setTheme
} from "$lib/api";
import { initialAppState, toCommandError, type AppState } from "./state";

function createAppStore() {
  const { subscribe, update } = writable<AppState>(initialAppState());

  // Déduplication pour éviter refreshes concurrents
  let refreshInProgress = false;
  let pendingRefresh = false;

  async function refreshSecondaryData() {
    if (refreshInProgress) {
      pendingRefresh = true;
      return;
    }
    refreshInProgress = true;
    pendingRefresh = false;
    try {
      const [settings, history, status] = await Promise.all([loadSettings(), listWeeks(), getAppStatus()]);
      update((state) => ({ ...state, settings, history, status }));
      if (pendingRefresh) {
        await refreshSecondaryData();
      }
    } finally {
      refreshInProgress = false;
    }
  }

  async function refreshHistoryOnly() {
    if (refreshInProgress) {
      pendingRefresh = true;
      return;
    }
    refreshInProgress = true;
    pendingRefresh = false;
    try {
      const history = await listWeeks();
      update((state) => ({ ...state, history }));
    } finally {
      refreshInProgress = false;
      if (pendingRefresh) {
        await refreshSecondaryData();
      }
    }
  }

  async function bootstrap() {
    update((state) => ({ ...state, loading: true, error: null }));

    try {
      const bootstrapState: BootstrapState = await loadBootstrap();
      update((state) => ({
        ...state,
        loading: false,
        bootstrapped: true,
        version: bootstrapState.version,
        configChecksum: bootstrapState.configChecksum,
        activeWeek: bootstrapState.activeWeek
      }));
      await refreshSecondaryData();
    } catch (error) {
      update((state) => ({ ...state, loading: false, error: toCommandError(error) }));
    }
  }

  async function persistWeek(input: SaveWeekInput) {
    update((state) => ({ ...state, savingWeek: true, error: null }));
    try {
      await saveWeek(input);
      // Recharger l'activeWeek pour obtenir les totaux recalculés par le backend
      const bootstrapState = await loadBootstrap();
      update((state) => ({
        ...state,
        activeWeek: bootstrapState.activeWeek,
        savingWeek: false,
      }));
      await refreshHistoryOnly();
    } catch (error) {
      update((state) => ({ ...state, savingWeek: false, error: toCommandError(error) }));
    }
  }

  async function switchWeek(weekStart: string) {
    update((state) => ({ ...state, switchingWeek: true, error: null }));
    try {
      const activeWeek = await createOrSwitchWeek({ weekStart });
      update((state) => ({ ...state, activeWeek, switchingWeek: false }));
      await refreshHistoryOnly();
    } catch (error) {
      update((state) => ({ ...state, switchingWeek: false, error: toCommandError(error) }));
    }
  }

  async function persistSettings(input: SaveSettingsInput) {
    update((state) => ({ ...state, savingSettings: true, error: null }));
    try {
      const settings = await saveSettings(input);
      update((state) => ({ ...state, settings, savingSettings: false }));
      await refreshSecondaryData();
    } catch (error) {
      update((state) => ({ ...state, savingSettings: false, error: toCommandError(error) }));
    }
  }

  async function changeTheme(theme: ThemePreference) {
    try {
      await setTheme({ theme });
      document.documentElement.dataset.theme = theme;
      await refreshSecondaryData();
    } catch (error) {
      update((state) => ({ ...state, error: toCommandError(error) }));
    }
  }

  async function removeWeek(weekId: string) {
    // Capture l'historique actuel avant modification (pour rollback en cas d'erreur)
    let originalHistory: WeekListItem[] = [];
    update((state) => {
      originalHistory = state.history;
      return {
        ...state,
        history: state.history.filter((item) => item.weekId !== weekId)
      };
    });

    try {
      await deleteWeek({ weekId });
      // Recharge seulement l'historique pour confirmer la suppression
      const history = await listWeeks();
      update((state) => ({ ...state, history }));
    } catch (error) {
      // En cas d'erreur, restaure l'historique original
      update((state) => ({
        ...state,
        history: originalHistory,
        error: toCommandError(error)
      }));
    }
  }

  async function loadAnalytics() {
    update((state) => ({ ...state, loading: true, error: null }));
    try {
      const analytics: AnalyticsDataView = await getAnalytics();
      update((state) => ({ ...state, analytics, loading: false }));
    } catch (error) {
      update((state) => ({ ...state, loading: false, error: toCommandError(error) }));
    }
  }

  return {
    subscribe,
    bootstrap,
    persistWeek,
    switchWeek,
    persistSettings,
    changeTheme,
    removeWeek,
    loadAnalytics
  };
}

export const appStore = createAppStore();
