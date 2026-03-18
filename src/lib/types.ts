export type ThemePreference = "light" | "dark";

export interface DayEntryView {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
  totalMinutes: number;
  totalLabel: string;
}

export interface DaySummaryView {
  dayId: number;
  label: string;
  workedMinutes: number;
  workedLabel: string;
}

export interface WeekSummaryView {
  totalMinutes: number;
  totalLabel: string;
  overtimeMinutes: number;
  overtimeLabel: string;
  averageMinutes: number;
  averageLabel: string;
  workedDays: number;
  longestDay: DaySummaryView | null;
  shortestDay: DaySummaryView | null;
  quickRead: string;
  percentage: number;
}

export interface WeekSheetView {
  weekId: string;
  weekStart: string;
  entries: DayEntryView[];
  overtimeThresholdMinutes: number;
  summary: WeekSummaryView;
}

export interface WeekListItem {
  weekId: string;
  weekStart: string;
  totalMinutes: number;
  totalLabel: string;
  workedDays: number;
  updatedAt: string;
}

export interface ConfiguredDayView {
  dayId: number;
  label: string;
  enabled: boolean;
}

export interface SettingsView {
  overtimeThresholdMinutes: number;
  overtimeThresholdLabel: string;
  theme: ThemePreference;
  defaultStart: string;
  defaultEnd: string;
  defaultBreak: string;
  configuredDays: ConfiguredDayView[];
  activeWeekId: string | null;
}

export interface ThemeView {
  theme: ThemePreference;
}

export interface BootstrapState {
  theme: ThemePreference;
  overtimeThresholdMinutes: number;
  activeWeek: WeekSheetView;
  configChecksum: string;
  version: string;
}

export interface AppStatusView {
  version: string;
  configChecksum: string;
  storageStatus: "healthy" | "degraded";
  latestMigrationStatus: string;
  activeWeekId: string | null;
  latestDiagnosticSnapshotId: string | null;
}

export interface SaveWeekDayEntryInput {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
}

export interface SaveWeekInput {
  weekId: string;
  weekStart: string;
  overtimeThresholdMinutes: number;
  entries: SaveWeekDayEntryInput[];
}

export interface WeekSelectorInput {
  weekStart: string;
}

export interface DeleteWeekInput {
  weekId: string;
}

export interface SaveSettingsInput {
  overtimeThresholdMinutes: number;
  defaultStart: string;
  defaultEnd: string;
  defaultBreak: string;
  configuredDays: ConfiguredDayView[];
}

export interface ThemeInput {
  theme: ThemePreference;
}

export interface CommandError {
  code: string;
  message: string;
  correlationId: string;
  retryable: boolean;
}

export interface DataExport {
  version: string;
  exportedAt: string;
  settings: SettingsView;
  weeks: WeekSheetView[];
}

// Analytics types
export interface DayOfWeekStats {
  dayIndex: number;
  dayName: string;
  entryCount: number;
  averageMinutes: number;
  averageLabel: string;
  totalMinutes: number;
  totalLabel: string;
}

export interface WeeklyTrendPoint {
  weekStart: string;
  totalMinutes: number;
  totalLabel: string;
  workedDays: number;
  overtimeMinutes: number;
  overtimeLabel: string;
}

export interface MonthlyStatsView {
  month: string;
  weeksCount: number;
  totalMinutes: number;
  totalLabel: string;
  weeklyAverageMinutes: number;
  weeklyAverageLabel: string;
}

export interface AnalyticsDataView {
  dayOfWeekStats: DayOfWeekStats[];
  weeklyTrends: WeeklyTrendPoint[];
  monthlyStats: MonthlyStatsView[];
  totalWeeks: number;
}
