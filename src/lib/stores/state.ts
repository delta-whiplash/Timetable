import type {
  AnalyticsDataView,
  AppStatusView,
  CommandError,
  SettingsView,
  WeekListItem,
  WeekSheetView
} from "$lib/types";

export interface AppState {
  bootstrapped: boolean;
  loading: boolean;
  savingWeek: boolean;
  savingSettings: boolean;
  error: CommandError | null;
  version: string;
  configChecksum: string;
  activeWeek: WeekSheetView | null;
  settings: SettingsView | null;
  history: WeekListItem[];
  status: AppStatusView | null;
  analytics: AnalyticsDataView | null;
}

export function initialAppState(): AppState {
  return {
    bootstrapped: false,
    loading: true,
    savingWeek: false,
    savingSettings: false,
    error: null,
    version: "",
    configChecksum: "",
    activeWeek: null,
    settings: null,
    history: [],
    status: null,
    analytics: null
  };
}

export function toCommandError(error: unknown): CommandError {
  if (typeof error === "object" && error !== null && "message" in error) {
    return {
      code: "frontend.invoke_failed",
      message: String((error as { message: unknown }).message),
      correlationId: crypto.randomUUID(),
      retryable: false
    };
  }

  return {
    code: "frontend.unknown",
    message: "Une erreur inattendue est survenue.",
    correlationId: crypto.randomUUID(),
    retryable: false
  };
}
