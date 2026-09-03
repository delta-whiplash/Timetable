# Task 5: Update BentoCard UI for Vacation

## Files
- Modify: `src/lib/components/BentoCard.svelte` (add vacation toggle)
- Test: Manual test in browser

## Interfaces
- Consumes: `DayType` from types.ts
- Produces: UI with vacation toggle button

## Steps

### Step 1: Add vacation toggle to BentoCard

```svelte
<!-- In src/lib/components/BentoCard.svelte, update script section -->
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
    const types: DayType[] = ["work", "vacation", "disabled"];
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
  $: isDisabled = !entry.enabled;
</script>
```

### Step 2: Update the header section with day type button

```svelte
<!-- Replace the existing header section -->
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
        disabled={disabled}
        on:click={cycleDayType}
        aria-label="Type: {entry.dayType}"
        title="Cliquez pour changer le type de jour"
      >
        {#if isVacation}
          🏖️
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
```

### Step 3: Update the article classes

```svelte
<!-- Update the article tag class binding -->
<article
  class="bento-card"
  class:bento-card--weekend={isWeekend}
  class:bento-card--disabled={isDisabled}
  class:bento-card--vacation={isVacation}
  role="region"
  aria-label={entry.label}
>
```

### Step 4: Add CSS styles

```css
/* Add to the style section at the end */
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

.bento-card--vacation {
  opacity: 0.85;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-primary-subtle) 100%);
  border-color: var(--color-primary);
}
```

### Step 5: Test manually in browser

Run: `pnpm run tauri:dev`
Expected: See vacation toggle button, clicking cycles through types

### Step 6: Commit

```bash
git add src/lib/components/BentoCard.svelte
git commit -m "feat(ui): add vacation day toggle with visual distinction"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-5-report.md` with:
1. Status: DONE or BLOCKED
2. Visual test results
3. Commits made
4. Any concerns or deviations from plan
