<script lang="ts">
  export let open = false;
  export let title = "Confirmer";
  export let message = "";
  export let confirmText = "Confirmer";
  export let cancelText = "Annuler";
  export let onConfirm: () => void = () => {};
  export let onCancel: () => void = () => {};

  function handleConfirm() {
    open = false;
    onConfirm();
  }

  function handleCancel() {
    open = false;
    onCancel();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleCancel();
    } else if (event.key === "Enter") {
      handleConfirm();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" on:click={handleBackdropClick} aria-label="Fermer">
    <div class="modal" class:modal--open={open} role="dialog" aria-modal="true" aria-labelledby="modal-title" tabindex="0" on:click={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2 id="modal-title">{title}</h2>
      </div>

      <div class="modal-body">
        <p>{message}</p>
      </div>

      <div class="modal-footer">
        <button class="button" type="button" on:click={handleCancel}>
          {cancelText}
        </button>
        <button class="button button--danger" type="button" on:click={handleConfirm}>
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 20px;
    backdrop-filter: blur(4px);
  }

  .modal {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    max-width: 380px;
    width: 100%;
    border: 1px solid var(--color-border);
    animation: modal-in 0.15s ease-out;
  }

  @keyframes modal-in {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(-10px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .modal-header {
    padding: var(--space-lg) var(--space-lg) var(--space-md);
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .modal-body {
    padding: var(--space-md) var(--space-lg) var(--space-sm);
  }

  .modal-body p {
    margin: 0;
    line-height: 1.5;
    color: var(--color-text-muted);
    white-space: pre-wrap;
  }

  .modal-footer {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
    padding: var(--space-md) var(--space-lg) var(--space-lg);
  }

  .button {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    padding: 10px 18px;
    border-radius: var(--radius-md);
    font-weight: 500;
    transition: all 0.15s ease;
    cursor: pointer;
  }

  .button:hover {
    background: var(--color-bg-hover);
    border-color: var(--color-border-strong);
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
