<script lang="ts">
  import TimeStepper from "$lib/components/TimeStepper.svelte";
  import type { DayEntryView } from "$lib/types";
  import { createEventDispatcher } from "svelte";

  export let entry: DayEntryView;
  export let disabled = false;

  // Default values from settings
  export let defaultStart: string = "08:00";
  export let defaultEnd: string = "18:00";
  export let defaultBreak: string = "01:00";

  const dispatch = createEventDispatcher();

  // Local reactive copy of break time in HH:MM format (for backend)
  let localBreakTime = entry.breakTime;

  // Format HH:MM to human readable "Xh Ymin"
  function formatToHuman(timeHHMM: string): string {
    const [hours, mins] = timeHHMM.split(":").map(Number);
    const totalMinutes = (hours || 0) * 60 + (mins || 0);
    if (totalMinutes === 0) return "0min";
    if (totalMinutes < 60) return `${totalMinutes}min`;
    const h = Math.floor(totalMinutes / 60);
    const m = totalMinutes % 60;
    return m > 0 ? `${h}h${m}min` : `${h}h`;
  }

  // Parse human format "1h30" or "30min" or "1h" to minutes
  function parseHumanFormat(value: string): number {
    value = value.toLowerCase().replace(/\s/g, "");

    // Match "1h30min" or "1h30" or "30min"
    const hourMatch = value.match(/(\d+)h/);
    const minMatch = value.match(/(\d+)min/);

    let hours = 0;
    let mins = 0;

    if (hourMatch) hours = parseInt(hourMatch[1]) || 0;
    if (minMatch) mins = parseInt(minMatch[1]) || 0;

    // If just a number with no h or min, treat as minutes
    if (!hourMatch && !minMatch) {
      const num = parseInt(value) || 0;
      if (num > 0) return num;
    }

    return hours * 60 + mins;
  }

  // Convert "HH:MM" to minutes for break time
  function breakToMinutes(breakTime: string): number {
    const [hours, mins] = breakTime.split(":").map(Number);
    return (hours ?? 0) * 60 + (mins ?? 0);
  }

  function timeToMinutes(time: string | null): number | null {
    if (!time) return null;
    const [hours, mins] = time.split(":").map(Number);
    return (hours ?? 0) * 60 + (mins ?? 0);
  }

  // Sync localBreakTime when entry changes
  $: if (entry.breakTime !== localBreakTime) {
    localBreakTime = entry.breakTime;
  }

  function toggleEnabled() {
    if (disabled) return;
    const wasEnabled = entry.enabled;
    entry.enabled = !entry.enabled;

    // Apply defaults when enabling
    if (!wasEnabled && entry.enabled) {
      if (!entry.start) entry.start = defaultStart;
      if (!entry.end) entry.end = defaultEnd;
      if (!entry.breakTime || entry.breakTime === "00:00" || entry.breakTime === "0") {
        localBreakTime = defaultBreak;
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
    let value = input.value.trim();

    if (value === "") {
      localBreakTime = "00:00";
    } else if (value.includes(":")) {
      // HH:MM format
      const [hours, mins] = value.split(":").map(Number);
      const totalMins = (hours || 0) * 60 + (mins || 0);
      if (totalMins >= 0 && totalMins <= 300) {
        localBreakTime = value.padStart(5, "0");
      }
    } else if (value.includes("h") || value.includes("min")) {
      // Human format "1h30" or "30min"
      const totalMins = parseHumanFormat(value);
      if (totalMins >= 0 && totalMins <= 300) {
        localBreakTime = minutesToTime(totalMins) ?? "00:00";
      }
    } else {
      // Just a number, treat as minutes
      const mins = parseInt(value) || 0;
      if (mins >= 0 && mins <= 300) {
        localBreakTime = minutesToTime(mins) ?? "00:00";
      }
    }
    entry.breakTime = localBreakTime;
    handleChange();
  }

  function adjustBreak(amount: number) {
    const current = localBreakTime.includes(":")
      ? breakToMinutes(localBreakTime)
      : parseInt(localBreakTime) || 0;
    const newBreak = Math.max(0, Math.min(300, current + amount));
    // Convert to HH:MM format for backend
    localBreakTime = minutesToTime(newBreak) ?? "00:00";
    entry.breakTime = localBreakTime;
    handleChange();
  }

  function handleChange() {
    dispatch("change", entry);
  }

  function minutesToTime(minutes: number | null): string | null {
    if (minutes === null) return null;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}`;
  }

  function formatTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours}h${mins.toString().padStart(2, "0")}`;
  }

  $: startMinutes = timeToMinutes(entry.start);
  $: endMinutes = timeToMinutes(entry.end);
  // Make breakMinutesValue directly depend on localBreakTime for reactivity
  $: breakMinutesValue = localBreakTime.includes(":")
    ? breakToMinutes(localBreakTime)
    : parseInt(localBreakTime) || 0;
  $: totalMinutes = entry.start && entry.end && entry.enabled && startMinutes !== null && endMinutes !== null
    ? Math.max(0, endMinutes - startMinutes - breakMinutesValue)
    : 0;
  $: totalLabel = totalMinutes > 0 ? formatTime(totalMinutes) : "—";
</script>

<div class="day-row {!entry.enabled ? 'day-row--disabled' : ''}">
  <!-- Ligne 1: Nom du jour + temps total -->
  <div class="day-row-header">
    <label class="day-row-toggle">
      <input type="checkbox" checked={entry.enabled} disabled={disabled} on:change={toggleEnabled} />
      <span class="day-row-label">{entry.label}</span>
    </label>
    <span class="day-row-total">{totalLabel}</span>
  </div>

  <!-- Ligne 2: Début et fin de journée -->
  <div class="day-row-worktime">
    <TimeStepper value={startMinutes} disabled={disabled || !entry.enabled} on:change={(e) => updateStart(e.detail)} />
    <TimeStepper value={endMinutes} disabled={disabled || !entry.enabled} on:change={(e) => updateEnd(e.detail)} />
  </div>

  <!-- Ligne 3: Pause -->
  <div class="day-row-break">
    <span class="break-label">Pause</span>
    <button
      class="break-btn"
      type="button"
      disabled={disabled || !entry.enabled}
      on:click={() => adjustBreak(-30)}
      aria-label="Diminuer la pause"
    >−</button>
    <input
      type="text"
      class="break-input"
      inputmode="numeric"
      placeholder="0min"
      value={formatToHuman(localBreakTime)}
      disabled={disabled || !entry.enabled}
      on:change={updateBreak}
    />
    <button
      class="break-btn"
      type="button"
      disabled={disabled || !entry.enabled}
      on:click={() => adjustBreak(30)}
      aria-label="Augmenter la pause"
    >+</button>
  </div>
</div>

<style>
  .day-row {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--space-sm);
    transition: all 0.15s ease;
  }

  .day-row:hover {
    border-color: var(--color-border-strong);
  }

  .day-row--disabled {
    opacity: 0.6;
    background: var(--color-bg-alt);
  }

  /* Ligne 1: Nom du jour + temps total */
  .day-row-header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .day-row-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
  }

  .day-row-toggle input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--color-primary);
  }

  .day-row-label {
    font-weight: 600;
    font-size: 0.9rem;
    min-width: 60px;
  }

  .day-row-total {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--color-primary);
    font-variant-numeric: tabular-nums;
    text-align: right;
    min-width: 60px;
  }

  /* Ligne 2: Début et fin de journée */
  .day-row-worktime {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  /* Ligne 3: Pause */
  .day-row-break {
    display: grid;
    grid-template-columns: auto var(--control-md) 1fr var(--control-md);
    gap: var(--space-xs);
    align-items: center;
  }

  .break-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .break-btn {
    height: var(--control-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-alt);
    border-radius: var(--radius-sm);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    color: var(--color-text);
  }

  .break-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
  }

  .break-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .break-input {
    width: 100%;
    height: var(--control-md);
    text-align: center;
    font-size: 0.85rem;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0 var(--space-sm);
    background: var(--color-bg-alt);
    color: var(--color-text);
    box-sizing: border-box;
  }

  .break-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .break-input:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 1px;
    border-color: var(--color-primary);
  }
</style>
