<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import BentoCard from "./BentoCard.svelte";
  import type { DayEntryView } from "$lib/types";

  export let entries: DayEntryView[] = [];
  export let disabled = false;
  export let defaultStart = "08:00";
  export let defaultEnd = "18:00";
  export let defaultBreak = "01:00";

  const dispatch = createEventDispatcher();

  function handleChange(event: CustomEvent<DayEntryView>) {
    dispatch("change", event.detail);
  }

  function handleCopyFromPrevious(dayId: number) {
    dispatch("copyPrevious", dayId);
  }

  // Split entries: Monday-Saturday (regular cards), Sunday (full-width or smaller)
  $: regularEntries = entries.filter((e) => e.dayId < 6);
  $: sundayEntry = entries.find((e) => e.dayId === 6);
</script>

<div class="bento-grid-container">
  <!-- Quick Actions Toolbar -->
  <div class="bento-toolbar">
    <span class="bento-toolbar-hint">
      Utilisez le bouton de copie sur chaque jour pour copier les horaires de la veille
    </span>
  </div>

  <!-- 3-Column Bento Grid -->
  <div class="bento-grid" aria-label="Grille horaire hebdomadaire">
    {#each regularEntries as entry (entry.dayId)}
      <BentoCard
        {entry}
        {disabled}
        {defaultStart}
        {defaultEnd}
        {defaultBreak}
        on:change={handleChange}
        on:copyPrevious={() => handleCopyFromPrevious(entry.dayId)}
      />
    {/each}

    {#if sundayEntry}
      <div class="bento-grid-sunday">
        <BentoCard
          entry={sundayEntry}
          {disabled}
          {defaultStart}
          {defaultEnd}
          {defaultBreak}
          on:change={handleChange}
          on:copyPrevious={() => handleCopyFromPrevious(sundayEntry.dayId)}
        />
      </div>
    {/if}
  </div>
</div>

<style>
  .bento-grid-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  /* Toolbar */
  .bento-toolbar {
    display: flex;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .bento-toolbar-hint {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-style: italic;
  }

  /* 3-Column Grid */
  .bento-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-md);
  }

  /* Sunday spans full width or 2 columns */
  .bento-grid-sunday {
    grid-column: 1 / -1;
  }

  /* Responsive: 2 columns on medium screens */
  @media (max-width: 1100px) {
    .bento-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  /* Responsive: 1 column on small screens */
  @media (max-width: 720px) {
    .bento-grid {
      grid-template-columns: 1fr;
    }

    .bento-toolbar {
      flex-direction: column;
      align-items: stretch;
    }

    .bento-toolbar-hint {
      text-align: center;
    }
  }
</style>
