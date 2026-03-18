<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let weekStart: string;
  export let disabled = false;

  const dispatch = createEventDispatcher();

  function getMondayOfWeek(dateStr: string): Date {
    const date = new Date(dateStr);
    const day = date.getDay();
    const diff = date.getDate() - day + (day === 0 ? -6 : 1);
    const monday = new Date(date.setDate(diff));
    monday.setHours(0, 0, 0, 0);
    return monday;
  }

  function formatDate(date: Date): string {
    return date.toISOString().slice(0, 10);
  }

  function getCurrentWeekMonday(): string {
    const now = new Date();
    const day = now.getDay();
    const diff = now.getDate() - day + (day === 0 ? -6 : 1);
    const monday = new Date(now.setDate(diff));
    monday.setHours(0, 0, 0, 0);
    return formatDate(monday);
  }

  function formatDisplayRange(weekStart: string): string {
    const monday = getMondayOfWeek(weekStart);
    const sunday = new Date(monday);
    sunday.setDate(sunday.getDate() + 6);

    const options: Intl.DateTimeFormatOptions = { day: "numeric", month: "short" };
    const formatter = new Intl.DateTimeFormat("fr-FR", options);

    return `${formatter.format(monday)} — ${formatter.format(sunday)}`;
  }

  function previousWeek() {
    if (disabled) return;
    const monday = getMondayOfWeek(weekStart);
    monday.setDate(monday.getDate() - 7);
    weekStart = formatDate(monday);
    dispatchChange();
  }

  function nextWeek() {
    if (disabled) return;
    const monday = getMondayOfWeek(weekStart);
    monday.setDate(monday.getDate() + 7);
    weekStart = formatDate(monday);
    dispatchChange();
  }

  function goToCurrentWeek() {
    if (disabled) return;
    weekStart = getCurrentWeekMonday();
    dispatchChange();
  }

  function dispatchChange() {
    dispatch("change", weekStart);
  }

  function handleDateInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    if (value) {
      weekStart = value;
      dispatchChange();
    }
  }

  $: displayRange = formatDisplayRange(weekStart);
  $: pickerValue = weekStart;
  $: isCurrentWeek = weekStart === getCurrentWeekMonday();
</script>

<div class="week-selector">
  <button
    type="button"
    class="week-selector-btn"
    on:click={previousWeek}
    disabled={disabled}
    aria-label="Semaine précédente"
  >
    ‹
  </button>
  <span class="week-selector-range">{displayRange}</span>
  <input
    type="date"
    class="week-selector-picker"
    bind:value={pickerValue}
    on:change={handleDateInput}
    disabled={disabled}
  />
  <button
    type="button"
    class="week-selector-btn"
    on:click={nextWeek}
    disabled={disabled}
    aria-label="Semaine suivante"
  >
    ›
  </button>
  <button
    type="button"
    class="week-selector-current"
    class:week-selector-current--active={isCurrentWeek}
    on:click={goToCurrentWeek}
    disabled={disabled || isCurrentWeek}
  >
    Aujourd'hui
  </button>
</div>

<style>
  .week-selector {
    display: grid;
    grid-template-columns: var(--control-md) 1fr auto var(--control-md) auto;
    gap: var(--space-sm);
    align-items: center;
    padding: var(--space-sm);
    background: var(--color-bg-alt);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .week-selector-btn {
    height: var(--control-md);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 1.2rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .week-selector-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
  }

  .week-selector-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .week-selector-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .week-selector-range {
    text-align: center;
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--color-text);
  }

  .week-selector-picker {
    width: 100%;
    min-width: 130px;
    height: var(--control-md);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0 var(--space-sm);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.85rem;
    font-family: inherit;
    box-sizing: border-box;
  }

  .week-selector-picker:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-subtle);
  }

  .week-selector-current {
    height: var(--control-md);
    padding: 0 var(--space-md);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.8rem;
    font-weight: 600;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .week-selector-current:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .week-selector-current:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .week-selector-current--active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .week-selector-current--active:hover:not(:disabled) {
    background: var(--color-primary-hover);
    border-color: var(--color-primary-hover);
  }
</style>
