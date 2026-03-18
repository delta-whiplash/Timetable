<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let value: number | null;
  export let disabled = false;
  export let step = 30;

  const dispatch = createEventDispatcher();

  function increment() {
    if (disabled) return;
    const current = value ?? 8 * 60; // Default 8:00
    const newValue = Math.min(current + step, 24 * 60 - step);
    dispatch("change", newValue);
  }

  function decrement() {
    if (disabled) return;
    const current = value ?? 8 * 60;
    const newValue = Math.max(current - step, 0);
    dispatch("change", newValue);
  }

  function formatTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}`;
  }

  $: displayValue = value !== null ? formatTime(value) : "--:--";
</script>

<div class="time-stepper {disabled ? 'time-stepper--disabled' : ''}">
  <button
    type="button"
    class="time-stepper-btn"
    on:click={decrement}
    disabled={disabled}
    aria-label="Moins 30 minutes"
  >
    −
  </button>
  <span class="time-stepper-value">{displayValue}</span>
  <button
    type="button"
    class="time-stepper-btn"
    on:click={increment}
    disabled={disabled}
    aria-label="Plus 30 minutes"
  >
    +
  </button>
</div>

<style>
  .time-stepper {
    display: grid;
    grid-template-columns: var(--control-md) 1fr var(--control-md);
    gap: 1px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--color-bg-alt);
    height: var(--control-md);
  }

  .time-stepper--disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .time-stepper-btn {
    height: 100%;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 1.1rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s;
  }

  .time-stepper-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .time-stepper-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .time-stepper-value {
    text-align: center;
    font-variant-numeric: tabular-nums;
    font-weight: 500;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-alt);
    height: 100%;
  }
</style>
