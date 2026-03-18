<script lang="ts">
  import { importData } from "$lib/api";

  let importing = false;
  let importError: string | null = null;
  let importSuccess = false;

  async function handleImport(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    importing = true;
    importError = null;
    importSuccess = false;

    try {
      const text = await file.text();
      await importData(text);
      importSuccess = true;
      alert("Import réussi ! L'application va recharger.");
      setTimeout(() => location.reload(), 1000);
    } catch (error) {
      console.error("Import error:", error);
      importError = "Erreur lors de l'import. Vérifiez que le fichier est valide.";
    } finally {
      importing = false;
      input.value = ""; // Reset file input
    }
  }
</script>

<section class="panel" aria-label="Gestion des données">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Données</p>
      <h2>Sauvegarde locale</h2>
    </div>
  </div>

  <div class="data-management">
    <p class="support">
      Vos données sont enregistrées localement sur cet ordinateur.
    </p>

    <div class="data-utilities">
      <details class="utilities-section">
        <summary class="utilities-toggle">Importer des données</summary>
        <div class="utilities-content">
          <p class="utilities-hint">
            Importez vos données depuis un fichier de sauvegarde.
          </p>

          <div class="data-actions">
            <label class="utility-button">
              Choisir un fichier
              <input
                type="file"
                accept=".json"
                disabled={importing}
                on:change={handleImport}
                style="display: none;"
              />
            </label>
          </div>

          {#if importError}
            <p class="error-message">{importError}</p>
          {/if}

          {#if importSuccess}
            <p class="success-message">Import réussi ! Rechargement...</p>
          {/if}
        </div>
      </details>
    </div>
  </div>
</section>

<style>
  .data-management {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .data-utilities {
    margin-top: 0.5rem;
  }

  .utilities-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .utilities-toggle {
    cursor: pointer;
    padding: 10px 14px;
    background: var(--color-bg-alt);
    color: var(--color-text-muted);
    font-size: 0.85rem;
    font-weight: 500;
    user-select: none;
    list-style: none;
    transition: color 120ms ease;
  }

  .utilities-toggle::-webkit-details-marker {
    display: none;
  }

  .utilities-toggle:hover {
    color: var(--color-text);
  }

  .utilities-toggle::after {
    content: "▸";
    float: right;
    transition: transform 120ms ease;
  }

  details[open] .utilities-toggle::after {
    transform: rotate(90deg);
  }

  .utilities-content {
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .utilities-hint {
    color: var(--color-text-muted);
    font-size: 0.85rem;
    margin: 0;
    line-height: 1.4;
  }

  .data-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .utility-button {
    border: 1px solid var(--color-border);
    background: var(--color-bg-alt);
    color: var(--color-text-muted);
    padding: 6px 12px;
    font-size: 0.85rem;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color 120ms ease, color 120ms ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .utility-button:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }

  .utility-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-message {
    padding: 0.75rem;
    background: #fee;
    border: 1px solid #f88;
    border-radius: 4px;
    color: #c00;
    font-size: 0.875rem;
  }

  :root[data-theme="dark"] .error-message {
    background: rgba(255, 127, 144, 0.15);
    border-color: var(--color-danger);
    color: #ffadbd;
  }

  .success-message {
    padding: 0.75rem;
    background: #efe;
    border: 1px solid #8f8;
    border-radius: 4px;
    color: #080;
    font-size: 0.875rem;
  }

  :root[data-theme="dark"] .success-message {
    background: rgba(118, 212, 167, 0.15);
    border-color: var(--color-success);
    color: #a3e9cc;
  }
</style>
