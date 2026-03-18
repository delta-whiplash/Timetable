<script lang="ts">
  import type { WeekListItem } from "$lib/types";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";

  export let items: WeekListItem[] = [];
  export let activeWeekId: string | null = null;
  export let loading = false;
  export let onSelect: (weekStart: string) => void;
  export let onDelete: (weekId: string) => void;

  let showDeleteModal = false;
  let weekToDelete: { id: string; start: string } | null = null;

  function openDeleteModal(weekId: string, weekStart: string) {
    weekToDelete = { id: weekId, start: weekStart };
    showDeleteModal = true;
  }

  function handleDeleteConfirm() {
    if (weekToDelete) {
      onDelete(weekToDelete.id);
      showDeleteModal = false;
      weekToDelete = null;
    }
  }

  function handleDeleteCancel() {
    showDeleteModal = false;
    weekToDelete = null;
  }
</script>

<section class="panel" aria-label="Historique">
  <div class="panel-heading">
    <div>
      <p class="eyebrow">Historique</p>
      <h2>Semaines enregistrées</h2>
    </div>
  </div>

  {#if loading}
    <p class="empty-state">Chargement de l'historique…</p>
  {:else if items.length === 0}
    <p class="empty-state">Aucune semaine enregistrée.</p>
  {:else}
    <div class="history-list">
      {#each items as item (item.weekId)}
        <article class:active={item.weekId === activeWeekId} class="history-item">
          <div>
            <strong>{item.weekStart}</strong>
            <p>{item.totalLabel} · {item.workedDays} jour(s)</p>
            <small>{item.updatedAt}</small>
          </div>

          <div class="history-actions">
            <button class="button" type="button" on:click={() => onSelect(item.weekStart)}>
              Ouvrir
            </button>
            <button
              class="button button--danger"
              type="button"
              on:click={() => openDeleteModal(item.weekId, item.weekStart)}
            >
              Supprimer
            </button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>

<ConfirmModal
  open={showDeleteModal}
  title="Supprimer la semaine"
  message={weekToDelete
    ? `Êtes-vous sûr de vouloir supprimer la semaine du ${weekToDelete.start} ?

Cette action est irréversible.`
    : ""}
  confirmText="Supprimer"
  cancelText="Annuler"
  onConfirm={handleDeleteConfirm}
  onCancel={handleDeleteCancel}
/>

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-heading {
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  .panel-heading .eyebrow {
    margin: 0;
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .panel-heading h2 {
    margin: var(--space-xs) 0 0;
    font-size: 1.25rem;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .empty-state {
    text-align: center;
    padding: var(--space-xl) var(--space-lg);
    color: var(--color-text-muted);
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: all 0.15s ease;
  }

  .history-item:hover {
    border-color: var(--color-border-strong);
    box-shadow: var(--shadow-sm);
  }

  .history-item.active {
    border-color: var(--color-primary);
    background: var(--color-primary-subtle);
  }

  .history-item strong {
    display: block;
    font-weight: 600;
    color: var(--color-text);
    margin-bottom: var(--space-xs);
  }

  .history-item p {
    margin: 0;
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .history-item small {
    display: block;
    margin-top: var(--space-xs);
    font-size: 0.75rem;
    color: var(--color-text-muted);
    opacity: 0.7;
  }

  .history-actions {
    display: flex;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  /* Beautiful buttons matching the modal style */
  .button {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    padding: 10px 18px;
    border-radius: var(--radius-md);
    font-weight: 500;
    font-size: 0.9rem;
    transition: all 0.15s ease;
    cursor: pointer;
    white-space: nowrap;
  }

  .button:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .button:active {
    transform: translateY(0);
  }

  .button--danger {
    background: var(--color-danger);
    color: white;
    border-color: var(--color-danger);
  }

  .button--danger:hover {
    background: var(--color-danger-hover);
    border-color: var(--color-danger-hover);
  }
</style>
