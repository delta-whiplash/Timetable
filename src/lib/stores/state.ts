import type {
  AnalyticsDataView,
  CommandError,
  SettingsView,
  WeekListItem,
  WeekSheetView
} from "$lib/types";

export interface AppState {
  bootstrapped: boolean;
  loading: boolean;
  savingWeek: boolean;
  switchingWeek: boolean;
  savingSettings: boolean;
  error: CommandError | null;
  activeWeek: WeekSheetView | null;
  settings: SettingsView | null;
  history: WeekListItem[];
  analytics: AnalyticsDataView | null;
}

const INITIAL_STATE: AppState = {
  bootstrapped: false,
  loading: true,
  savingWeek: false,
  switchingWeek: false,
  savingSettings: false,
  error: null,
  activeWeek: null,
  settings: null,
  history: [],
  analytics: null
};

export function initialAppState(): AppState {
  return INITIAL_STATE;
}

export function toCommandError(error: unknown): CommandError {
  if (typeof error === "object" && error !== null && "message" in error) {
    return {
      code: "frontend.invoke_failed",
      message: String((error as { message: unknown }).message),
      correlationId: crypto.randomUUID()
    };
  }

  return {
    code: "frontend.unknown",
    message: "Une erreur inattendue est survenue.",
    correlationId: crypto.randomUUID()
  };
}
