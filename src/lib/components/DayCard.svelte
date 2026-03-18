<script lang="ts">
  import type { DayEntryView } from "$lib/types";
  import TimeStepper from "$lib/components/TimeStepper.svelte";

  export let entry: DayEntryView;
  export let disabled = false;
  export let onChange: (next: DayEntryView) => void;

  // Default values in minutes
  const DEFAULT_START_MINUTES = 8 * 60; // 08:00
  const DEFAULT_END_MINUTES = 18 * 60; // 18:00
  const DEFAULT_BREAK_MINUTES = 60; // 01:00

  // Convert "HH:MM" string to minutes
  function timeToMinutes(time: string | null): number | null {
    if (!time) return null;
    const [hours, minutes] = time.split(":").map(Number);
    return hours * 60 + minutes;
  }

  // Convert minutes to "HH:MM" string
  function minutesToTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}`;
  }

  function updateField<K extends keyof DayEntryView>(key: K, value: DayEntryView[K]) {
    onChange({ ...entry, [key]: value });
  }

  function updateStart(newMinutes: number) {
    updateField("start", minutesToTime(newMinutes));
  }

  function updateEnd(newMinutes: number) {
    updateField("end", minutesToTime(newMinutes));
  }

  function updateBreak(newMinutes: number) {
    updateField("breakTime", minutesToTime(newMinutes));
  }

  function handleEnabledChange(checked: boolean) {
    if (checked && !entry.enabled) {
      // When enabling, set default values if not already set
      onChange({
        ...entry,
        enabled: true,
        start: entry.start || minutesToTime(DEFAULT_START_MINUTES),
        end: entry.end || minutesToTime(DEFAULT_END_MINUTES),
        breakTime: entry.breakTime || minutesToTime(DEFAULT_BREAK_MINUTES),
      });
    } else {
      updateField("enabled", checked);
    }
  }

  // Get current values in minutes for TimeStepper
  $: startMinutes = timeToMinutes(entry.start);
  $: endMinutes = timeToMinutes(entry.end);
  $: breakMinutes = timeToMinutes(entry.breakTime) ?? 0;
</script>

<article class="day-card">
  <header class="day-header">
    <div class="day-name">
      {entry.label}
    </div>

    <div class="day-header-actions">
      <label class="toggle">
        <input
          type="checkbox"
          checked={entry.enabled}
          disabled={disabled}
          on:change={(event) => handleEnabledChange(event.currentTarget.checked)}
        />
        <span>{entry.enabled ? "Actif" : "Inactif"}</span>
      </label>

      <span class="pill">{entry.totalLabel}</span>
    </div>
  </header>

  <div class="inputs-grid">
    <label class="field">
      <span>Début</span>
      <TimeStepper
        value={startMinutes}
        disabled={disabled || !entry.enabled}
        step={30}
        on:change={(event) => updateStart(event.detail)}
      />
    </label>

    <label class="field">
      <span>Fin</span>
      <TimeStepper
        value={endMinutes}
        disabled={disabled || !entry.enabled}
        step={30}
        on:change={(event) => updateEnd(event.detail)}
      />
    </label>

    <label class="field">
      <span>Pause</span>
      <TimeStepper
        value={breakMinutes}
        disabled={disabled || !entry.enabled}
        step={30}
        on:change={(event) => updateBreak(event.detail)}
      />
    </label>
  </div>
</article>
