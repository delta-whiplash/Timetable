<script lang="ts">
  import { appStore } from "$lib/stores/app";
  import type { AppState } from "$lib/stores/state";
  import type { ThemePreference } from "$lib/types";

  export let activeTab: "timesheet" | "history" | "analytics" | "settings" = "timesheet";

  let state: AppState;

  appStore.subscribe((value) => {
    state = value;
  });

  function setTab(tab: typeof activeTab) {
    activeTab = tab;
  }

  function toggleTheme() {
    const newTheme: ThemePreference = state.settings?.theme === "dark" ? "light" : "dark";
    appStore.changeTheme(newTheme);
  }

  const tabs = [
    { id: "timesheet" as const, label: "Feuille de temps", icon: "📅" },
    { id: "history" as const, label: "Historique", icon: "📋" },
    { id: "analytics" as const, label: "Analytiques", icon: "📊" },
    { id: "settings" as const, label: "Configuration", icon: "⚙️" },
  ];
</script>

<aside class="sidebar">
  <!-- KPI Card unifie : Total + Objectif -->
  {#if state.activeWeek}
    <div class="sidebar-card sidebar-card--total">
      <span class="sidebar-card-label">Semaine active</span>
      <span class="sidebar-card-value">{state.activeWeek.summary.totalLabel}</span>

      <!-- Objectif -->
      <div class="kpi-target">
        <span class="kpi-target-label">Objectif</span>
        <span class="kpi-target-value">{state.settings?.overtimeThresholdLabel ?? "35h 00min"}</span>
      </div>

      <!-- Barre de progression avec couleur d'alerte si depassement -->
      <div class="progress-bar">
        <div
          class="progress-bar-fill {state.activeWeek.summary.percentage > 100 ? 'progress-bar-fill--warning' : ''}"
          style="width: {Math.min(state.activeWeek.summary.percentage, 100)}%"
        ></div>
      </div>

      <!-- Pourcentage et depassement -->
      <div class="kpi-footer">
        <span class="kpi-percent">{state.activeWeek.summary.percentage}%</span>
        {#if state.activeWeek.summary.percentage > 100}
          <span class="kpi-overtime">+{state.activeWeek.summary.percentage - 100}%</span>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Navigation -->
  <nav class="sidebar-nav">
    {#each tabs as tab}
      <button
        type="button"
        class="sidebar-nav-item {activeTab === tab.id ? 'sidebar-nav-item--active' : ''}"
        on:click={() => setTab(tab.id)}
      >
        <span class="sidebar-nav-icon">{tab.icon}</span>
        <span class="sidebar-nav-label">{tab.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Actions footer -->
  <div class="sidebar-footer">
    <button type="button" class="sidebar-action" on:click={toggleTheme}>
      {state.settings?.theme === "dark" ? "☀️" : "🌙"}
      <span>{state.settings?.theme === "dark" ? "Mode clair" : "Mode sombre"}</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    padding: 0;
    height: 100%;
  }

  .sidebar-card {
    background: var(--color-bg-alt);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 14px 16px;
    text-align: center;
  }

  .sidebar-card--total {
    background: linear-gradient(135deg, var(--color-primary-light) 0%, var(--color-bg-alt) 100%);
    border-color: var(--color-primary);
  }

  .sidebar-card-label {
    display: block;
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    margin-bottom: 6px;
  }

  .sidebar-card-value {
    display: block;
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: -0.04em;
    color: var(--color-text);
  }

  .kpi-target {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--space-sm);
    padding-top: var(--space-sm);
    border-top: 1px solid var(--color-border);
  }

  .kpi-target-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .kpi-target-value {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .progress-bar {
    height: 8px;
    background: var(--color-bg);
    border-radius: 999px;
    overflow: hidden;
    margin-top: var(--space-sm);
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--color-primary);
    transition: width 300ms ease;
  }

  .progress-bar-fill--warning {
    background: linear-gradient(90deg, #f97316 0%, #ef4444 100%);
  }

  .kpi-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--space-sm);
  }

  .kpi-percent {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .kpi-overtime {
    font-size: 0.8rem;
    font-weight: 700;
    color: #f97316;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .sidebar-nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    transition: all 150ms ease;
    text-align: left;
  }

  .sidebar-nav-item:hover {
    background: var(--color-bg-alt);
    color: var(--color-text);
    border: 1px solid var(--color-border-strong);
  }

  .sidebar-nav-item:active {
    transform: scale(0.98);
  }

  .sidebar-nav-item--active {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
  }

  .sidebar-nav-icon {
    font-size: 1.1rem;
  }

  .sidebar-nav-label {
    flex: 1;
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-top: var(--space-sm);
    border-top: 1px solid var(--color-border);
  }

  .sidebar-action {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    transition: all 150ms ease;
    text-align: left;
  }

  .sidebar-action:hover {
    background: var(--color-bg-alt);
    color: var(--color-text);
    border: 1px solid var(--color-border-strong);
  }
</style>
