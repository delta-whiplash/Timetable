<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import BentoGrid from "$lib/components/BentoGrid.svelte";
  import WeekSelector from "$lib/components/WeekSelector.svelte";
  import AnalyticsPanel from "$lib/components/AnalyticsPanel.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import HistoryPanel from "$lib/components/HistoryPanel.svelte";
  import { appStore } from "$lib/stores/app";
  import { initialAppState, type AppState } from "$lib/stores/state";
  import type {
    DayEntryView,
    SaveSettingsInput,
    ThemePreference,
    WeekSheetView
  } from "$lib/types";

  let state: AppState = initialAppState();
  let activeTab: "timesheet" | "history" | "analytics" | "settings" = "timesheet";

  // Debounce pour l'autosave
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let hasPendingChanges = false;

  const unsubscribe = appStore.subscribe((value: AppState) => {
    state = value;
    if (value.settings) {
      document.documentElement.dataset.theme = value.settings.theme;
    }
  });

  onMount(() => {
    void appStore.bootstrap();
  });

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout); // Cleanup du timer
    unsubscribe();
  });

  function saveDraft(entries: DayEntryView[], immediate = false) {
    const activeWeek = state.activeWeek;
    if (!activeWeek) return;

    const input = {
      weekId: activeWeek.weekId,
      weekStart: activeWeek.weekStart,
      overtimeThresholdMinutes: activeWeek.overtimeThresholdMinutes,
      entries: entries.map((entry) => ({
        dayId: entry.dayId,
        label: entry.label,
        enabled: entry.enabled,
        start: entry.start,
        end: entry.end,
        breakTime: entry.breakTime
      }))
    };

    if (immediate) {
      // Save immédiat (sans debounce)
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = null;
      hasPendingChanges = false;
      void appStore.persistWeek(input);
    } else {
      // Save avec debounce
      hasPendingChanges = true;
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = setTimeout(() => {
        saveTimeout = null;
        hasPendingChanges = false;
        void appStore.persistWeek(input);
      }, 2500);
    }
  }

  function updateEntry(next: DayEntryView) {
    const activeWeek = state.activeWeek;
    if (!activeWeek) return;
    const entries = activeWeek.entries.map((entry: DayEntryView) =>
      entry.dayId === next.dayId ? next : entry
    );
    state = { ...state, activeWeek: { ...activeWeek, entries } as WeekSheetView };
    saveDraft(entries);
  }

  // Force le save immédiat des modifications en attente
  function flushPendingChanges() {
    if (hasPendingChanges && state.activeWeek) {
      saveDraft(state.activeWeek.entries, true);
    }
  }

  function handleWeekChange(weekStart: string) {
    flushPendingChanges(); // Save immédiat avant changement
    void appStore.switchWeek(weekStart);
  }

  function handleOpenFromHistory(weekStart: string) {
    flushPendingChanges(); // Save immédiat avant changement
    void appStore.switchWeek(weekStart);
    activeTab = "timesheet"; // Basculer vers l'onglet timesheet
  }

  function setTab(tab: typeof activeTab) {
    flushPendingChanges(); // Save immédiat avant changement d'onglet
    activeTab = tab;
  }
</script>

<svelte:head>
  <title>Timetable Desktop</title>
</svelte:head>

{#if state.loading && !state.bootstrapped}
  <main class="shell shell--centered">
    <section class="loading-panel">
      <p class="eyebrow">Initialisation</p>
      <h1>Chargement de l'application…</h1>
    </section>
  </main>
{:else}
  <main class="shell">
    {#if state.error}
      <section class="alert">
        <strong>{state.error.message}</strong>
        <span>Code: {state.error.code} · Corrélation: {state.error.correlationId}</span>
      </section>
    {/if}

    <div class="app-layout">
      <!-- Sidebar -->
      <Sidebar bind:activeTab={activeTab} />

      <!-- Main Content -->
      <main class="main-content">
        {#if state.activeWeek}
          <!-- Header with Week Selector -->
          <header class="content-header">
            <div>
              <p class="eyebrow">Feuille de temps</p>
              <h1>Semaine active</h1>
              <span class="save-indicator" class:show={hasPendingChanges}>● Enregistrement...</span>
            </div>
            <WeekSelector
              weekStart={state.activeWeek.weekStart}
              disabled={state.savingWeek}
              on:change={(e) => handleWeekChange(e.detail)}
            />
          </header>

          {#if activeTab === "timesheet"}
            <!-- Timesheet View - Bento Grid Layout -->
            <div class="content-body">
              <BentoGrid
                entries={state.activeWeek.entries}
                disabled={state.savingWeek}
                defaultStart={state.settings?.defaultStart ?? "08:00"}
                defaultEnd={state.settings?.defaultEnd ?? "18:00"}
                defaultBreak={state.settings?.defaultBreak ?? "01:00"}
                on:change={(e) => updateEntry(e.detail)}
                on:copyPrevious={(e) => {
                  // Copy from previous day (dayId - 1) to current day
                  const targetDayId = e.detail;
                  if (!state.activeWeek) return;
                  const sourceEntry = state.activeWeek.entries.find(
                    (entry: DayEntryView) => entry.dayId === targetDayId - 1
                  );
                  if (sourceEntry) {
                    const updated = state.activeWeek.entries.map((entry: DayEntryView) =>
                      entry.dayId === targetDayId
                        ? { ...entry, start: sourceEntry.start, end: sourceEntry.end, breakTime: sourceEntry.breakTime }
                        : entry
                    );
                    state = { ...state, activeWeek: { ...state.activeWeek, entries: updated } as WeekSheetView };
                    saveDraft(updated);
                  }
                }}
              />
            </div>

          {:else if activeTab === "history"}
            <!-- History View -->
            <div class="content-body content-body--full">
              <header class="content-header">
                <h1>Semaines précédentes</h1>
              </header>
              <HistoryPanel
                items={state.history}
                activeWeekId={state.activeWeek?.weekId ?? null}
                loading={state.loading}
                onSelect={handleOpenFromHistory}
                onDelete={(weekId: string) => appStore.removeWeek(weekId)}
              />
            </div>

          {:else if activeTab === "analytics"}
            <!-- Analytics View -->
            <div class="content-body content-body--full">
              <AnalyticsPanel />
            </div>

          {:else if activeTab === "settings"}
            <!-- Settings View -->
            <div class="content-body">
              <header class="content-header">
                <h1>Configuration</h1>
              </header>
              {#if state.settings}
                <SettingsPanel
                  settings={state.settings}
                  saving={state.savingSettings}
                  onSave={(input: SaveSettingsInput) => appStore.persistSettings(input)}
                  onThemeChange={(theme: ThemePreference) => appStore.changeTheme(theme)}
                />
              {/if}
            </div>
          {/if}
        {/if}
      </main>
    </div>
  </main>
{/if}

<style>
  .shell {
    min-height: 100vh;
    padding: var(--space-md);
  }

  .shell--centered {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: var(--space-md);
    max-width: 1400px;
    margin: 0 auto;
    height: calc(100vh - 32px);
  }

  .main-content {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .content-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) 20px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-sm);
  }

  .content-header h1 {
    margin: 0;
    font-size: 1.3rem;
    letter-spacing: -0.03em;
  }

  .save-indicator {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    margin-top: var(--space-xs);
    visibility: hidden;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .save-indicator.show {
    visibility: visible;
    opacity: 1;
  }

  .save-indicator::before {
    content: "●";
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .content-body {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .content-body--full {
    overflow-y: visible;
  }

  .loading-panel {
    text-align: center;
  }

  .alert {
    max-width: 1400px;
    margin: 0 auto var(--space-md);
    display: grid;
    gap: var(--space-sm);
    padding: var(--space-md) 20px;
    background: var(--color-surface);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, var(--color-border));
    border-radius: var(--radius-lg);
  }

  @media (max-width: 900px) {
    .app-layout {
      grid-template-columns: 1fr;
      height: auto;
    }

  }
</style>
