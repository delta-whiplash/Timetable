<script lang="ts">
  import DayTableRow from "./DayTableRow.svelte";
  import type { DayEntryView } from "$lib/types";
  import { createEventDispatcher } from "svelte";

  export let entries: DayEntryView[] = [];
  export let disabled = false;
  export let defaultStart = "08:00";
  export let defaultEnd = "18:00";
  export let defaultBreak = "01:00";
  export let showToolbar = true;

  const dispatch = createEventDispatcher();

  function handleChange(event: CustomEvent<DayEntryView>) {
    dispatch("change", event.detail);
  }

  function handleApplyTemplate(event: CustomEvent<DayEntryView>) {
    dispatch("applyTemplate", event.detail);
  }

  function applyToAllEnabled() {
    entries.forEach((entry) => {
      if (entry.enabled) {
        entry.start = defaultStart;
        entry.end = defaultEnd;
        entry.breakTime = defaultBreak;
        dispatch("change", entry);
      }
    });
  }
</script>

<div class="timesheet-table-container">
  {#if showToolbar}
    <div class="timesheet-toolbar">
      <button
        type="button"
        class="timesheet-toolbar-btn"
        disabled={disabled}
        on:click={applyToAllEnabled}
        title="Appliquer les horaires par defaut a tous les jours actifs"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
          <line x1="16" y1="2" x2="16" y2="6"></line>
          <line x1="8" y1="2" x2="8" y2="6"></line>
          <line x1="3" y1="10" x2="21" y2="10"></line>
        </svg>
        <span>Journee type</span>
      </button>
    </div>
  {/if}

  <div class="timesheet-table-wrapper">
    <table class="timesheet-table">
      <thead class="timesheet-thead">
        <tr class="timesheet-header-row">
          <th class="timesheet-header-cell timesheet-header-cell--checkbox">
            <span class="sr-only">Actif</span>
          </th>
          <th class="timesheet-header-cell timesheet-header-cell--label">Jour</th>
          <th class="timesheet-header-cell timesheet-header-cell--time">Debut</th>
          <th class="timesheet-header-cell timesheet-header-cell--time">Fin</th>
          <th class="timesheet-header-cell timesheet-header-cell--break">Pause</th>
          <th class="timesheet-header-cell timesheet-header-cell--total">Total</th>
          <th class="timesheet-header-cell timesheet-header-cell--actions">
            <span class="sr-only">Actions</span>
          </th>
        </tr>
      </thead>
      <tbody class="timesheet-tbody">
        {#each entries as entry (entry.dayId)}
          <DayTableRow
            {entry}
            {disabled}
            {defaultStart}
            {defaultEnd}
            defaultBreak={defaultBreak}
            on:change={handleChange}
            on:applyTemplate={handleApplyTemplate}
          />
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .timesheet-table-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .timesheet-toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .timesheet-toolbar-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 8px 12px;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text);
    background: var(--color-bg-alt);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .timesheet-toolbar-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
  }

  .timesheet-toolbar-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .timesheet-toolbar-btn svg {
    width: 16px;
    height: 16px;
  }

  .timesheet-table-wrapper {
    overflow-x: auto;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .timesheet-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  .timesheet-thead {
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .timesheet-header-row {
    background: var(--color-bg-alt);
    border-bottom: 2px solid var(--color-border);
  }

  .timesheet-header-cell {
    padding: var(--space-sm) var(--space-sm);
    text-align: left;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-muted);
  }

  .timesheet-header-cell--checkbox {
    width: 32px;
    text-align: center;
  }

  .timesheet-header-cell--label {
    width: 80px;
  }

  .timesheet-header-cell--time {
    width: 70px;
  }

  .timesheet-header-cell--break {
    width: 60px;
  }

  .timesheet-header-cell--total {
    width: 60px;
    text-align: right;
  }

  .timesheet-header-cell--actions {
    width: 36px;
    text-align: center;
  }

  .timesheet-tbody {
    background: var(--color-surface);
  }

  /* Screen reader only class */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
