export type ThemePreference = "light" | "dark";

export interface DayEntryView {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
  hasDepartureDeduction: boolean;
  hasReturnDeduction: boolean;
  totalMinutes: number;
  totalLabel: string;
}

export interface WeekSummaryView {
  totalLabel: string;
  percentage: number;
  cumulativeBalanceMinutes: number;
  cumulativeBalanceLabel: string;
}

export interface WeekSheetView {
  weekId: string | null;
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

export interface BootstrapState {
  activeWeek: WeekSheetView;
}

export interface SaveWeekDayEntryInput {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
  hasDepartureDeduction: boolean;
  hasReturnDeduction: boolean;
}

export interface SaveWeekInput {
  weekId: string | null;
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

export interface CommandError {
  code: string;
  message: string;
  correlationId: string;
}

// Analytics types
export interface DayOfWeekStats {
  dayName: string;
  averageMinutes: number;
}

export interface WeeklyTrendPoint {
  weekStart: string;
  totalMinutes: number;
}

export interface MonthlyStatsView {
  month: string;
  totalLabel: string;
  weeklyAverageLabel: string;
}

export interface AnalyticsDataView {
  dayOfWeekStats: DayOfWeekStats[];
  weeklyTrends: WeeklyTrendPoint[];
  monthlyStats: MonthlyStatsView[];
  totalWeeks: number;
}
