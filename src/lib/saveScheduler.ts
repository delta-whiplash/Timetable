/**
 * Ordonnanceur d'autosave : debounce court (600 ms par défaut) avec re-armement,
 * flush immédiat sérialisé et annulation propre à la destruction.
 *
 * L'objectif est une sauvegarde invisible pour l'utilisateur : le save part tôt
 * (debounce court + flush sur blur/changement de contexte) sans jamais perdre
 * la dernière valeur saisie, et deux saves ne se chevauchent jamais.
 */
export interface SaveSchedulerOptions<T> {
  save: (input: T) => Promise<void>;
  /** Délai de debounce en ms (défaut 600). */
  delayMs?: number;
  /** Notifié à true quand des modifications sont en attente, false sinon. */
  onPendingChange?: (pending: boolean) => void;
}

export class SaveScheduler<T> {
  private readonly save: (input: T) => Promise<void>;
  private readonly delayMs: number;
  private readonly onPendingChange?: (pending: boolean) => void;

  private timeout: ReturnType<typeof setTimeout> | null = null;
  private pendingInput: T | null = null;
  private inFlight: Promise<void> | null = null;

  constructor(options: SaveSchedulerOptions<T>) {
    this.save = options.save;
    this.delayMs = options.delayMs ?? 600;
    this.onPendingChange = options.onPendingChange;
  }

  get hasPending(): boolean {
    return this.pendingInput !== null;
  }

  /** Programme un save debounced ; chaque appel ré-arme le timer et remplace la valeur en attente. */
  schedule(input: T): void {
    this.pendingInput = input;
    this.setPending(true);
    if (this.timeout) clearTimeout(this.timeout);
    this.timeout = setTimeout(() => {
      this.timeout = null;
      void this.flush();
    }, this.delayMs);
  }

  /** Sauvegarde immédiatement la dernière valeur en attente (si présente), sérialisé avec tout save en cours. */
  flush(): Promise<void> {
    const input = this.pendingInput;
    if (input === null) return this.inFlight ?? Promise.resolve();
    this.pendingInput = null;
    this.setPending(false);
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    // Démarre immédiatement si aucun save n'est en cours, sinon enchaîne derrière.
    const run = () => this.save(input);
    this.inFlight = this.inFlight ? this.inFlight.then(run, run) : run();
    return this.inFlight;
  }

  /** Annule le timer sans sauvegarder (destruction du composant). */
  destroy(): void {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    this.pendingInput = null;
    this.setPending(false);
  }

  private setPending(pending: boolean): void {
    this.onPendingChange?.(pending);
  }
}
