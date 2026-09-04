<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { DayEntryView, DayType } from "$lib/types";
  import { toMinutes, toHHMM } from "$lib/time";

  export let entry: DayEntryView;
  export let disabled = false;
  export let defaultStart = "08:00";
  export let defaultEnd = "18:00";
  export let defaultBreak = "01:00";
  export let enableTravelDeduction = true;
  export let travelDeductionMinutes = 30;

  const dispatch = createEventDispatcher();

  function updateField<K extends keyof DayEntryView>(key: K, value: DayEntryView[K]) {
    dispatch("change", { ...entry, [key]: value });
  }

  function handleEnabledChange(checked: boolean) {
    if (checked && !entry.enabled) {
      dispatch("change", {
        ...entry,
        enabled: true,
        start: entry.start || defaultStart,
        end: entry.end || defaultEnd,
        breakTime: entry.breakTime || defaultBreak,
        dayType: "work" as DayType,
      });
    } else {
      updateField("enabled", checked);
    }
  }

  function cycleDayType() {
    const types: DayType[] = ["work", "vacation", "public_holiday", "disabled"];
    const currentIndex = types.indexOf(entry.dayType);
    const nextIndex = (currentIndex + 1) % types.length;
    const newType = types[nextIndex];

    if (newType === "disabled") {
      dispatch("change", { ...entry, enabled: false, dayType: "disabled" });
    } else {
      dispatch("change", {
        ...entry,
        enabled: true,
        dayType: newType,
        start: entry.start || defaultStart,
        end: entry.end || defaultEnd,
        breakTime: entry.breakTime || defaultBreak,
      });
    }
  }

  function updateTime(field: "start" | "end" | "breakTime", direction: 1 | -1) {
    const step = field === "breakTime" ? 15 : 30;
    const defaultValue = field === "breakTime" ? defaultBreak : defaultStart;
    const currentMinutes = toMinutes(entry[field]) ?? toMinutes(defaultValue) ?? 0;
    const newMinutes = Math.max(0, Math.min(24 * 60, currentMinutes + direction * step));
    updateField(field, toHHMM(newMinutes));
  }

  $: isWeekend = entry.dayId >= 5;
  $: isVacation = entry.dayType === "vacation";
  $: isPublicHoliday = entry.dayType === "public_holiday";
  $: isDisabled = !entry.enabled;
</script>

<article
  class="bento-card"
  class:bento-card--weekend={isWeekend}
  class:bento-card--disabled={isDisabled}
  class:bento-card--vacation={isVacation}
  class:bento-card--public-holiday={isPublicHoliday}
  role="region"
  aria-label={entry.label}
>
  <header class="bento-card-header">
    <div class="bento-card-day">
      <input
        type="checkbox"
        class="bento-card-checkbox"
        checked={entry.enabled}
        disabled={disabled}
        on:change={(e) => handleEnabledChange(e.currentTarget.checked)}
        aria-label="Activer {entry.label}"
      />
      <span class="bento-card-day-label">{entry.label}</span>
      {#if entry.enabled}
        <button
          type="button"
          class="day-type-btn"
          class:day-type-btn--vacation={isVacation}
          class:day-type-btn--public-holiday={isPublicHoliday}
          disabled={disabled}
          on:click={cycleDayType}
          aria-label="Type: {entry.dayType}"
          title="Cliquez pour changer le type de jour"
        >
          {#if isVacation}
            🏖️
          {:else if isPublicHoliday}
            🎉
          {:else}
            💼
          {/if}
        </button>
      {/if}
    </div>
    <div class="bento-card-total">
      {entry.totalLabel}
    </div>
  </header>

  <div class="bento-card-body">
    <div class="bento-card-time">
      <span class="bento-card-time-label">Début</span>
      <div class="stepper">
        <button
          type="button"
          class="stepper-btn stepper-btn--minus"
          disabled={disabled || !entry.enabled}
          on:click={() => updateTime("start", -1)}
          aria-label="Début -30 minutes"
        >−</button>
        <span class="stepper-value">
          {entry.start ?? "--:--"}
        </span>
        <button
          type="button"
          class="stepper-btn stepper-btn--plus"
          disabled={disabled || !entry.enabled}
          on:click={() => updateTime("start", 1)}
          aria-label="Début +30 minutes"
        >+</button>
      </div>
      {#if enableTravelDeduction}
	      <label class="bento-card-deplacement">
	        <input
	          type="checkbox"
	          class="bento-card-deplacement-checkbox"
	          checked={entry.hasDepartureDeduction}
	          disabled={disabled || !entry.enabled}
	          on:change={(e) => updateField("hasDepartureDeduction", e.currentTarget.checked)}
	        />
	        <span class="bento-card-deplacement-label">Départ {travelDeductionMinutes}min</span>
	      </label>
	    {/if}
    </div>

    <div class="bento-card-time">
      <span class="bento-card-time-label">Fin</span>
      <div class="stepper">
        <button
          type="button"
          class="stepper-btn stepper-btn--minus"
          disabled={disabled || !entry.enabled}
          on:click={() => updateTime("end", -1)}
          aria-label="Fin -30 minutes"
        >−</button>
        <span class="stepper-value">
          {entry.end ?? "--:--"}
        </span>
        <button
          type="button"
          class="stepper-btn stepper-btn--plus"
          disabled={disabled || !entry.enabled}
          on:click={() => updateTime("end", 1)}
          aria-label="Fin +30 minutes"
        >+</button>
      </div>
      {#if enableTravelDeduction}
        <label class="bento-card-deplacement">
          <input
            type="checkbox"
            class="bento-card-deplacement-checkbox"
            checked={entry.hasReturnDeduction}
            disabled={disabled || !entry.enabled}
            on:change={(e) => updateField("hasReturnDeduction", e.currentTarget.checked)}
          />
          <span class="bento-card-deplacement-label">Retour {travelDeductionMinutes}min</span>
        </label>
      {/if}
    </div>
  </div>

  <footer class="bento-card-footer">
    <span class="bento-card-pause-label">Pause</span>
    <div class="stepper stepper--compact">
      <button
        type="button"
        class="stepper-btn stepper-btn--minus"
        disabled={disabled || !entry.enabled}
        on:click={() => updateTime("breakTime", -1)}
        aria-label="Pause -15 minutes"
      >−</button>
      <span class="stepper-value">
        {entry.breakTime ?? "--:--"}
      </span>
      <button
        type="button"
        class="stepper-btn stepper-btn--plus"
        disabled={disabled || !entry.enabled}
        on:click={() => updateTime("breakTime", 1)}
        aria-label="Pause +15 minutes"
      >+</button>
    </div>
  </footer>
</article>

<style>
  .bento-card {
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: all 0.2s ease;
  }

  .bento-card:hover {
    border-color: var(--color-border-strong);
    box-shadow: var(--shadow-sm);
    transform: translateY(-1px);
  }

  .bento-card--disabled {
    opacity: 0.5;
    background: var(--color-bg-alt);
  }

  .bento-card--weekend {
    font-size: 0.9em;
  }

  /* Header */
  .bento-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-alt);
  }

  .bento-card-day {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .bento-card-checkbox {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--color-primary);
    transition: transform 0.1s ease;
  }

  .bento-card-checkbox:active:not(:disabled) {
    transform: scale(0.9);
  }

  .bento-card-checkbox:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .bento-card-day-label {
    font-weight: 600;
    color: var(--color-text);
    font-size: 0.95rem;
  }

  .bento-card-total {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--color-primary);
    font-variant-numeric: tabular-nums;
    transition: transform 0.1s ease;
  }

  /* Body */
  .bento-card-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-sm);
    padding: var(--space-md);
  }

  .bento-card-time {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .bento-card-time-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .bento-card-deplacement {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 4px;
    cursor: pointer;
  }

  .bento-card-deplacement-checkbox {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--color-primary);
  }

  .bento-card-deplacement-checkbox:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .bento-card-deplacement-label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  /* Footer */
  .bento-card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-alt);
  }

  .bento-card-pause-label {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    font-weight: 500;
  }

  /* Stepper */
  .stepper {
    display: inline-flex;
    align-items: center;
    height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    overflow: hidden;
  }

  .stepper--compact {
    height: 28px;
  }

  .stepper-btn {
    width: 28px;
    height: 100%;
    flex-shrink: 0;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 1rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s ease;
    padding: 0;
  }

  .stepper--compact .stepper-btn {
    width: 24px;
    font-size: 0.85rem;
  }

  .stepper-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .stepper-btn:active:not(:disabled) {
    transform: scale(0.9);
    background: var(--color-primary-subtle);
  }

  .stepper-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .stepper-value {
    min-width: 52px;
    text-align: center;
    font-variant-numeric: tabular-nums;
    font-weight: 500;
    font-size: 0.85rem;
    color: var(--color-text);
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease;
  }

  .stepper--compact .stepper-value {
    min-width: 48px;
    font-size: 0.8rem;
  }

  /* Weekend adjustments */
  .bento-card--weekend .bento-card-header {
    padding: var(--space-xs) var(--space-sm);
  }

  .bento-card--weekend .bento-card-body {
    padding: var(--space-sm);
  }

  .bento-card--weekend .bento-card-footer {
    padding: var(--space-xs) var(--space-sm);
  }

  .bento-card--weekend .bento-card-total {
    font-size: 1rem;
  }

  .bento-card--weekend .stepper {
    height: 28px;
  }

  .bento-card--weekend .stepper-btn {
    width: 24px;
    font-size: 0.85rem;
  }

  .bento-card--weekend .stepper-value {
    min-width: 48px;
    font-size: 0.8rem;
  }

  .day-type-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    cursor: pointer;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
    margin-left: auto;
  }

  .day-type-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
  }

  .day-type-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .day-type-btn--vacation {
    background: var(--color-primary-subtle);
    border-color: var(--color-primary);
  }

  .day-type-btn--public-holiday {
    background: var(--color-success-subtle, #d4edda);
    border-color: var(--color-success, #28a745);
  }

  .bento-card--vacation {
    opacity: 0.85;
    background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-primary-subtle) 100%);
    border-color: var(--color-primary);
  }

  .bento-card--public-holiday {
    opacity: 0.85;
    background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-success-subtle, #d4edda) 100%);
    border-color: var(--color-success, #28a745);
  }
</style>
