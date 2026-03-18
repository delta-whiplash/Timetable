<script lang="ts">
  import type { WeekSummaryView } from "$lib/types";

  export let summary: WeekSummaryView;
</script>

<section class="summary-grid" aria-label="Synthèse hebdomadaire">
  <article class="summary-card summary-card--hero">
    <p class="eyebrow">Total semaine</p>
    <strong class="hero-value">{summary.totalLabel}</strong>
    <p class="support">{summary.totalMinutes} minutes travaillées</p>
    <p class="support">{summary.quickRead}</p>
  </article>

  <article class="summary-card">
    <p class="eyebrow">Heures sup</p>
    <strong class="metric">{summary.overtimeLabel}</strong>
    <p class="support">Seuil hebdomadaire appliqué</p>
  </article>

  <article class="summary-card">
    <p class="eyebrow">Moyenne / jour</p>
    <strong class="metric">{summary.averageLabel}</strong>
    <p class="support">{summary.workedDays} jour(s) actif(s)</p>
  </article>
</section>
