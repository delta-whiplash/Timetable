import { invoke } from "@tauri-apps/api/core";
import type {
  AnalyticsDataView,
  AppStatusView,
  BootstrapState,
  DataExport,
  DeleteWeekInput,
  SaveSettingsInput,
  SaveWeekInput,
  SettingsView,
  ThemeInput,
  ThemeView,
  WeekListItem,
  WeekSelectorInput,
  WeekSheetView
} from "./types";

export function loadBootstrap(): Promise<BootstrapState> {
  return invoke("load_bootstrap");
}

export function getActiveWeek(): Promise<WeekSheetView> {
  return invoke("get_active_week");
}

export function saveWeek(input: SaveWeekInput): Promise<WeekSheetView> {
  return invoke("save_week", { input });
}

export function createOrSwitchWeek(input: WeekSelectorInput): Promise<WeekSheetView> {
  return invoke("create_or_switch_week", { input });
}

export function listWeeks(): Promise<WeekListItem[]> {
  return invoke("list_weeks");
}

export function deleteWeek(input: DeleteWeekInput): Promise<void> {
  return invoke("delete_week", { input });
}

export function loadSettings(): Promise<SettingsView> {
  return invoke("load_settings");
}

export function saveSettings(input: SaveSettingsInput): Promise<SettingsView> {
  return invoke("save_settings", { input });
}

export function setTheme(input: ThemeInput): Promise<ThemeView> {
  return invoke("set_theme", { input });
}

export function getAppStatus(): Promise<AppStatusView> {
  return invoke("get_app_status");
}

export function exportData(): Promise<string> {
  return invoke("export_data");
}

export function importData(jsonData: string): Promise<BootstrapState> {
  return invoke("import_data", { jsonData });
}

export function getAnalytics(): Promise<AnalyticsDataView> {
  return invoke("get_analytics");
}
