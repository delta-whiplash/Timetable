<script lang="ts">
  import BentoCard from "./BentoCard.svelte";
  import type { DayEntryView } from "$lib/types";

  export let entries: DayEntryView[] = [];
  export let disabled = false;
  export let defaultStart = "08:00";
  export let defaultEnd = "18:00";
  export let defaultBreak = "01:00";

  // Split entries: Monday-Saturday (regular cards), Sunday (full-width)
  $: regularEntries = entries.filter((e) => e.dayId < 6);
  $: sundayEntry = entries.find((e) => e.dayId === 6);
</script>

<div class="bento-grid" aria-label="Grille horaire hebdomadaire">
  {#each regularEntries as entry (entry.dayId)}
    <BentoCard
      {entry}
      {disabled}
      {defaultStart}
      {defaultEnd}
      {defaultBreak}
      on:change
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
        on:change
      />
    </div>
  {/if}
</div>

<style>
  .bento-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-md);
  }

  .bento-grid-sunday {
    grid-column: 1 / -1;
  }

  @media (max-width: 1100px) {
    .bento-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 720px) {
    .bento-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
