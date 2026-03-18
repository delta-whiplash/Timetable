<script lang="ts">
  import CompactTimeInput from "./CompactTimeInput.svelte";
  import type { DayEntryView } from "$lib/types";
  import { createEventDispatcher } from "svelte";

  export let entry: DayEntryView;
  export let disabled = false;
  export let defaultStart = "08:00";
  export let defaultEnd = "18:00";
  export let defaultBreak = "01:00";

  const dispatch = createEventDispatcher();

  function timeToMinutes(time: string | null): number | null {
    if (!time) return null;
    const [hours, mins] = time.split(":").map(Number);
    return (hours ?? 0) * 60 + (mins ?? 0);
  }

  function minutesToTime(minutes: number | null): string | null {
    if (minutes === null) return null;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}`;
  }

  function toggleEnabled() {
    if (disabled) return;
    const wasEnabled = entry.enabled;
    entry.enabled = !entry.enabled;

    if (!wasEnabled && entry.enabled) {
      if (!entry.start) entry.start = defaultStart;
      if (!entry.end) entry.end = defaultEnd;
      if (!entry.breakTime || entry.breakTime === "00:00") {
        entry.breakTime = defaultBreak;
      }
    }

    handleChange();
  }

  function updateStart(value: number | null) {
    entry.start = minutesToTime(value);
    handleChange();
  }

  function updateEnd(value: number | null) {
    entry.end = minutesToTime(value);
    handleChange();
  }

  function updateBreak(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    entry.breakTime = input.value;
    handleChange();
  }

  function handleChange() {
    dispatch("change", entry);
  }

  function applyTemplate() {
    dispatch("applyTemplate", entry);
  }

  // Check if this is a weekend day (dayId 6 = Saturday, 7 = Sunday)
  $: isWeekend = entry.dayId >= 6;

  // Check if overtime (more than 8 hours = 480 minutes)
  $: isOvertime = entry.totalMinutes > 480;

  $: startMinutes = timeToMinutes(entry.start);
  $: endMinutes = timeToMinutes(entry.end);
</script>

<tr
  class="day-table-row {!entry.enabled ? 'day-table-row--disabled' : ''} {isWeekend
    ? 'day-table-row--weekend'
    : ''}"
>
  <!-- Checkbox -->
  <td class="day-table-row-cell day-table-row-cell--checkbox">
    <input
      type="checkbox"
      class="day-table-row-checkbox"
      checked={entry.enabled}
      disabled={disabled}
      on:change={toggleEnabled}
      aria-label="Activer {entry.label}"
    />
  </td>

  <!-- Day Label -->
  <td class="day-table-row-cell day-table-row-cell--label">
    <span class="day-label">{entry.label}</span>
  </td>

  <!-- Start Time -->
  <td class="day-table-row-cell day-table-row-cell--time">
    <CompactTimeInput
      value={startMinutes}
      disabled={disabled || !entry.enabled}
      on:change={(e) => updateStart(e.detail)}
    />
  </td>

  <!-- End Time -->
  <td class="day-table-row-cell day-table-row-cell--time">
    <CompactTimeInput
      value={endMinutes}
      disabled={disabled || !entry.enabled}
      on:change={(e) => updateEnd(e.detail)}
    />
  </td>

  <!-- Break Time -->
  <td class="day-table-row-cell day-table-row-cell--break">
    <input
      type="text"
      class="break-input"
      inputmode="numeric"
      value={entry.breakTime}
      disabled={disabled || !entry.enabled}
      on:change={updateBreak}
      aria-label="Pause pour {entry.label}"
    />
  </td>

  <!-- Total -->
  <td class="day-table-row-cell day-table-row-cell--total">
    <span class="total-value {isOvertime ? 'total-value--overtime' : ''}">
      {entry.totalLabel}
    </span>
  </td>

  <!-- Actions -->
  <td class="day-table-row-cell day-table-row-cell--actions">
    <button
      type="button"
      class="action-btn"
      disabled={disabled || !entry.enabled}
      on:click={applyTemplate}
      aria-label="Appliquer la journee type a {entry.label}"
      title="Appliquer la journee type"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="16" y1="2" x2="16" y2="6"></line>
        <line x1="8" y1="2" x2="8" y2="6"></line>
        <line x1="3" y1="10" x2="21" y2="10"></line>
      </svg>
    </button>
  </td>
</tr>

<style>
  .day-table-row {
    transition: background-color 0.15s ease, opacity 0.15s ease;
    height: 48px;
  }

  .day-table-row:hover {
    background: var(--color-bg-alt);
  }

  .day-table-row--weekend {
    height: 32px;
    opacity: 0.7;
  }

  .day-table-row--disabled {
    opacity: 0.5;
  }

  .day-table-row--disabled:hover {
    background: transparent;
  }

  .day-table-row-cell {
    padding: 0 var(--space-sm);
    border-bottom: 1px solid var(--color-border);
    vertical-align: middle;
  }

  .day-table-row-cell--checkbox {
    width: 32px;
    text-align: center;
  }

  .day-table-row-checkbox {
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: var(--color-primary);
  }

  .day-table-row-cell--label {
    width: 80px;
  }

  .day-label {
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--color-text);
  }

  .day-table-row--weekend .day-label {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .day-table-row-cell--time {
    width: 70px;
  }

  .day-table-row-cell--break {
    width: 60px;
  }

  .break-input {
    width: 100%;
    height: 26px;
    text-align: center;
    font-size: 0.8rem;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-text);
    border-radius: var(--radius-sm);
    padding: 0 4px;
    transition: all 0.15s ease;
  }

  .break-input:hover:not(:disabled) {
    background: var(--color-bg-alt);
    border-color: var(--color-border);
  }

  .break-input:focus {
    outline: none;
    background: var(--color-bg-alt);
    border-color: var(--color-primary);
  }

  .break-input:disabled {
    cursor: not-allowed;
  }

  .day-table-row-cell--total {
    width: 60px;
    text-align: right;
  }

  .total-value {
    font-weight: 700;
    font-size: 0.95rem;
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
  }

  .total-value--overtime {
    color: #f97316; /* Orange for overtime */
  }

  .day-table-row--weekend .total-value {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .day-table-row-cell--actions {
    width: 36px;
    text-align: center;
  }

  .action-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
    padding: 0;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--color-bg-alt);
    color: var(--color-primary);
  }

  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .action-btn svg {
    width: 14px;
    height: 14px;
  }
</style>
