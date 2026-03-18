<script lang="ts">
  import type { AppStatusView } from "$lib/types";

  export let status: AppStatusView | null = null;
</script>

<section class="panel" aria-label="État applicatif">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Diagnostic</p>
      <h2>État applicatif</h2>
    </div>
  </div>

  {#if status}
    <dl class="diagnostics-grid">
      <div>
        <dt>Version</dt>
        <dd>{status.version}</dd>
      </div>
      <div>
        <dt>Checksum config</dt>
        <dd>{status.configChecksum}</dd>
      </div>
      <div>
        <dt>Stockage</dt>
        <dd>{status.storageStatus}</dd>
      </div>
      <div>
        <dt>Migrations</dt>
        <dd>{status.latestMigrationStatus}</dd>
      </div>
      <div>
        <dt>Semaine active</dt>
        <dd>{status.activeWeekId ?? "Aucune"}</dd>
      </div>
      <div>
        <dt>Dernier snapshot</dt>
        <dd>{status.latestDiagnosticSnapshotId ?? "Aucun"}</dd>
      </div>
    </dl>
  {:else}
    <p class="empty-state">État non disponible.</p>
  {/if}
</section>
