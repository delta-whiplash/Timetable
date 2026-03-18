<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let value: number | null = null;
  export let disabled = false;
  export let step = 15; // 15 minutes steps

  const dispatch = createEventDispatcher();

  let isEditing = false;
  let inputValue = "";

  function formatTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}`;
  }

  function parseTime(input: string): number | null {
    const trimmed = input.trim().toLowerCase();

    // HH:MM format (e.g., "08:30", "8:30")
    const hhmmMatch = trimmed.match(/^(\d{1,2}):(\d{2})$/);
    if (hhmmMatch) {
      const hours = parseInt(hhmmMatch[1]) || 0;
      const mins = parseInt(hhmmMatch[2]) || 0;
      if (hours >= 0 && hours < 24 && mins >= 0 && mins < 60) {
        return hours * 60 + mins;
      }
    }

    // "HhMM" format (e.g., "8h30", "14h00")
    const hMatch = trimmed.match(/^(\d{1,2})h(\d{2})?$/);
    if (hMatch) {
      const hours = parseInt(hMatch[1]) || 0;
      const mins = hMatch[2] ? parseInt(hMatch[2]) : 0;
      if (hours >= 0 && hours < 24 && mins >= 0 && mins < 60) {
        return hours * 60 + mins;
      }
    }

    // Decimal hours format (e.g., "8.5" = 8:30)
    const decimalMatch = trimmed.match(/^(\d+(?:\.\d+)?)$/);
    if (decimalMatch) {
      const decimal = parseFloat(decimalMatch[1]);
      if (decimal >= 0 && decimal < 24) {
        return Math.round(decimal * 60);
      }
    }

    // Compact format (e.g., "830" = 8:30, "1430" = 14:30)
    const compactMatch = trimmed.match(/^(\d{1,4})$/);
    if (compactMatch) {
      const num = parseInt(compactMatch[1]);
      if (num < 100) {
        // Single/double digit treated as hours
        return num * 60;
      } else {
        // Last two digits are minutes
        const hours = Math.floor(num / 100);
        const mins = num % 100;
        if (hours >= 0 && hours < 24 && mins >= 0 && mins < 60) {
          return hours * 60 + mins;
        }
      }
    }

    return null;
  }

  function increment() {
    if (disabled) return;
    const current = value ?? 8 * 60;
    const newValue = Math.min(current + step, 24 * 60 - step);
    dispatch("change", newValue);
  }

  function decrement() {
    if (disabled) return;
    const current = value ?? 8 * 60;
    const newValue = Math.max(current - step, 0);
    dispatch("change", newValue);
  }

  function handleFocus() {
    if (disabled) return;
    isEditing = true;
    inputValue = value !== null ? formatTime(value) : "";
  }

  function handleBlur() {
    if (!isEditing) return;
    isEditing = false;

    const parsed = parseTime(inputValue);
    if (parsed !== null) {
      dispatch("change", parsed);
    } else {
      inputValue = value !== null ? formatTime(value) : "";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      (event.currentTarget as HTMLInputElement).blur();
    } else if (event.key === "Escape") {
      inputValue = value !== null ? formatTime(value) : "";
      isEditing = false;
    }
  }

  $: displayValue = value !== null ? formatTime(value) : "--:--";
</script>

<div class="compact-time-input {disabled ? 'compact-time-input--disabled' : ''}" class:editing={isEditing}>
  <button
    type="button"
    class="compact-time-input-btn"
    on:click={decrement}
    disabled={disabled}
    aria-label="Moins 15 minutes"
  >
    −
  </button>

  <input
    type="text"
    class="compact-time-input-value"
    class:editing={isEditing}
    value={isEditing ? inputValue : displayValue}
    {disabled}
    aria-label="Saisir l'heure"
    inputmode="numeric"
    on:focus={handleFocus}
    on:blur={handleBlur}
    on:keydown={handleKeydown}
    on:input={(e) => inputValue = e.currentTarget.value}
    readonly={!isEditing}
  />

  <button
    type="button"
    class="compact-time-input-btn"
    on:click={increment}
    disabled={disabled}
    aria-label="Plus 15 minutes"
  >
    +
  </button>
</div>

<style>
  .compact-time-input {
    display: inline-flex;
    align-items: center;
    position: relative;
    font-variant-numeric: tabular-nums;
    height: 28px;
  }

  .compact-time-input--disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .compact-time-input-btn {
    position: absolute;
    width: 16px;
    height: 16px;
    border: none;
    background: var(--color-bg-alt);
    color: var(--color-text-muted);
    font-size: 0.75rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s ease, background 0.15s ease;
    z-index: 1;
  }

  .compact-time-input:hover .compact-time-input-btn {
    opacity: 1;
  }

  .compact-time-input-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .compact-time-input-btn:first-child {
    left: 2px;
  }

  .compact-time-input-btn:last-child {
    right: 2px;
  }

  .compact-time-input-value {
    width: 52px;
    height: 100%;
    text-align: center;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text);
    background: transparent;
    border: none;
    padding: 0;
    cursor: text;
    font-variant-numeric: tabular-nums;
  }

  .compact-time-input-value.editing {
    background: var(--color-bg-alt);
    border-radius: var(--radius-sm);
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  .compact-time-input-value:focus {
    outline: none;
  }

  .compact-time-input.editing .compact-time-input-btn {
    opacity: 0;
  }

  .compact-time-input-value[readonly] {
    cursor: default;
  }

  .compact-time-input-value:not([readonly]):hover {
    background: var(--color-bg-alt);
    border-radius: var(--radius-sm);
  }
</style>
