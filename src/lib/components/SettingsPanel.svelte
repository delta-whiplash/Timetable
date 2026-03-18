<script lang="ts">
  import type { ConfiguredDayView, SaveSettingsInput, SettingsView, ThemePreference } from "$lib/types";

  export let settings: SettingsView;
  export let saving = false;
  export let onSave: (input: SaveSettingsInput) => void;
  export let onThemeChange: (theme: ThemePreference) => void;

  // Overtime threshold in minutes
  let overtimeMinutes = settings.overtimeThresholdMinutes;
  // Default times in minutes
  let defaultStartMinutes = parseTimeToMinutes(settings.defaultStart);
  let defaultEndMinutes = parseTimeToMinutes(settings.defaultEnd);
  let defaultBreakMinutes = parseTimeToMinutes(settings.defaultBreak);

  // Parse "HH:MM" to minutes since midnight
  function parseTimeToMinutes(time: string): number {
    const [hours, minutes] = time.split(":").map(Number);
    return (hours || 0) * 60 + (minutes || 0);
  }

  // Format minutes to "HH:MM"
  function formatMinutesToTime(minutes: number): string {
    const h = Math.floor(Math.abs(minutes) / 60) % 24;
    const m = Math.abs(minutes) % 60;
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
  }

  // Format minutes to human-readable "Xh Ymin"
  function formatMinutesToHuman(minutes: number): string {
    if (minutes < 60) {
      return `${minutes} min`;
    }
    return `${Math.floor(minutes / 60)}h ${minutes % 60}min`;
  }

  function updateDay(index: number, next: ConfiguredDayView) {
    settings.configuredDays[index] = next;
    settings = { ...settings, configuredDays: [...settings.configuredDays] };
  }

  function submit() {
    onSave({
      overtimeThresholdMinutes: overtimeMinutes,
      defaultStart: formatMinutesToTime(defaultStartMinutes),
      defaultEnd: formatMinutesToTime(defaultEndMinutes),
      defaultBreak: formatMinutesToTime(defaultBreakMinutes),
      configuredDays: settings.configuredDays
    });
  }

  function adjustOvertime(amount: number) {
    overtimeMinutes = Math.max(15, Math.min(2880, overtimeMinutes + amount));
  }

  function adjustDefaultStart(amount: number) {
    defaultStartMinutes = (defaultStartMinutes + amount + 1440) % 1440;
  }

  function adjustDefaultEnd(amount: number) {
    defaultEndMinutes = (defaultEndMinutes + amount + 1440) % 1440;
  }

  function adjustDefaultBreak(amount: number) {
    defaultBreakMinutes = Math.max(0, Math.min(180, defaultBreakMinutes + amount));
  }
</script>

<section class="panel" aria-label="Paramètres">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Paramètres</p>
      <h2>Règles globales</h2>
    </div>
    <button class="save-button" type="button" disabled={saving} on:click={submit}>
      {saving ? "Enregistrement..." : "Enregistrer"}
    </button>
  </div>

  <div class="settings-grid">
    <!-- Objectif hebdomadaire avec contrôles souris -->
    <div class="field field--stepper">
      <span>Objectif hebdomadaire</span>
      <div class="stepper">
        <button class="stepper-btn" type="button" on:click={() => adjustOvertime(-30)} aria-label="Diminuer de 30 minutes">−</button>
        <span class="stepper-value">{formatMinutesToHuman(overtimeMinutes)}</span>
        <button class="stepper-btn" type="button" on:click={() => adjustOvertime(30)} aria-label="Augmenter de 30 minutes">+</button>
      </div>
    </div>

    <!-- Heure de début par défaut -->
    <div class="field field--stepper">
      <span>Heure de début par défaut</span>
      <div class="stepper">
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultStart(-30)} aria-label="Diminuer de 30 minutes">−</button>
        <span class="stepper-value">{formatMinutesToTime(defaultStartMinutes)}</span>
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultStart(30)} aria-label="Augmenter de 30 minutes">+</button>
      </div>
    </div>

    <!-- Heure de fin par défaut -->
    <div class="field field--stepper">
      <span>Heure de fin par défaut</span>
      <div class="stepper">
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultEnd(-30)} aria-label="Diminuer de 30 minutes">−</button>
        <span class="stepper-value">{formatMinutesToTime(defaultEndMinutes)}</span>
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultEnd(30)} aria-label="Augmenter de 30 minutes">+</button>
      </div>
    </div>

    <!-- Pause par défaut -->
    <div class="field field--stepper">
      <span>Pause midi par défaut</span>
      <div class="stepper">
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultBreak(-30)} aria-label="Diminuer de 30 minutes">−</button>
        <span class="stepper-value">{formatMinutesToHuman(defaultBreakMinutes)}</span>
        <button class="stepper-btn" type="button" on:click={() => adjustDefaultBreak(30)} aria-label="Augmenter de 30 minutes">+</button>
      </div>
    </div>

    <fieldset class="theme-switcher">
      <legend class="field-label">Thème</legend>
      <div class="segmented">
        <button
          class:active={settings.theme === "light"}
          type="button"
          on:click={() => onThemeChange("light")}
        >
          Clair
        </button>
        <button
          class:active={settings.theme === "dark"}
          type="button"
          on:click={() => onThemeChange("dark")}
        >
          Sombre
        </button>
      </div>
    </fieldset>
  </div>

  <div class="settings-days">
    <h3>Jours de travail</h3>
    <div class="settings-days-grid">
      {#each settings.configuredDays as day, index (day.dayId)}
        <button
          class="day-button {day.enabled ? 'day-button--active' : ''}"
          type="button"
          on:click={() => updateDay(index, { ...day, enabled: !day.enabled })}
        >
          <span class="day-label">{day.label}</span>
          <span class="day-status">{day.enabled ? "Travaille" : "Repos"}</span>
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .field--stepper {
    align-items: center;
  }

  .stepper {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .stepper-btn {
    width: 40px;
    height: 40px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    border-radius: var(--radius-md);
    font-size: 1.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .stepper-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
  }

  .stepper-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .stepper-value {
    min-width: 80px;
    text-align: center;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  .settings-days {
    margin-top: 1.5rem;
  }

  .settings-days h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-md);
  }

  .settings-days-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
    gap: var(--space-sm);
  }

  /* Boutons pour les jours - même style que l'historique */
  .day-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: var(--radius-md);
    font-weight: 500;
    font-size: 0.85rem;
    transition: all 0.15s ease;
    cursor: pointer;
    white-space: nowrap;
  }

  .day-button:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .day-button:active {
    transform: translateY(0);
  }

  /* Variant : jour actif (TRAVAILLE) */
  .day-button--active {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }

  .day-button--active:hover {
    background: var(--color-primary-hover);
    border-color: var(--color-primary-hover);
  }

  .day-label {
    font-weight: 600;
    font-size: 1rem;
  }

  .day-status {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.8;
  }

  .day-button--active .day-status {
    opacity: 1;
  }

  /* Bouton Enregistrer - même style que les boutons de l'historique */
  .save-button {
    border: 1px solid var(--color-border);
    background: var(--color-primary);
    color: white;
    padding: 10px 18px;
    border-radius: var(--radius-md);
    font-weight: 500;
    font-size: 0.9rem;
    transition: all 0.15s ease;
    cursor: pointer;
    white-space: nowrap;
    border-color: var(--color-primary);
  }

  .save-button:hover:not(:disabled) {
    background: var(--color-primary-hover);
    border-color: var(--color-primary-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .save-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .save-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .theme-switcher {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .theme-switcher legend {
    font-weight: 500;
  }

  .segmented {
    display: flex;
    background: var(--color-bg-alt);
    padding: 0.25rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
  }

  .segmented button {
    flex: 1;
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.15s;
  }

  .segmented button:hover {
    background: var(--color-bg-hover);
  }

  .segmented button.active {
    background: var(--color-surface);
    color: var(--color-primary);
    box-shadow: var(--shadow-sm);
  }
</style>
