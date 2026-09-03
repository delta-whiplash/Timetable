import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { SaveScheduler } from "./saveScheduler";

describe("SaveScheduler", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("ne sauvegarde pas avant le debounce et sauvegarde avec la dernière entrée", () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save });

    scheduler.schedule("a");
    vi.advanceTimersByTime(599);
    expect(save).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith("a");
  });

  it("ré-arme le debounce à chaque frappe et n'utilise que la dernière valeur", () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save });

    scheduler.schedule("a");
    vi.advanceTimersByTime(400);
    scheduler.schedule("b");
    vi.advanceTimersByTime(400);
    expect(save).not.toHaveBeenCalled();

    vi.advanceTimersByTime(200);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith("b");
  });

  it("flush sauvegarde immédiatement et annule le timer (pas de double save)", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save });

    scheduler.schedule("a");
    const flushed = scheduler.flush();
    expect(save).toHaveBeenCalledWith("a");

    await flushed;
    vi.advanceTimersByTime(1000);
    expect(save).toHaveBeenCalledTimes(1);
    expect(scheduler.hasPending).toBe(false);
  });

  it("flush sans modification en attente ne déclenche pas de save", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save });

    await scheduler.flush();
    expect(save).not.toHaveBeenCalled();
  });

  it("notifie le changement d'état pending pour l'indicateur UI", () => {
    const onPendingChange = vi.fn();
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save, onPendingChange });

    scheduler.schedule("a");
    expect(onPendingChange).toHaveBeenLastCalledWith(true);

    vi.advanceTimersByTime(600);
    expect(onPendingChange).toHaveBeenLastCalledWith(false);
  });

  it("destroy annule le timer sans sauvegarder", () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const scheduler = new SaveScheduler({ save });

    scheduler.schedule("a");
    scheduler.destroy();
    vi.advanceTimersByTime(2000);
    expect(save).not.toHaveBeenCalled();
    expect(scheduler.hasPending).toBe(false);
  });

  it("sérialise les saves : un flush pendant un save en cours attend le précédent", async () => {
    let resolveFirst: () => void;
    const firstSave = new Promise<void>((r) => (resolveFirst = r));
    const save = vi
      .fn()
      .mockImplementationOnce(() => firstSave)
      .mockResolvedValueOnce(undefined);
    const scheduler = new SaveScheduler({ save });

    scheduler.schedule("a");
    const first = scheduler.flush();
    scheduler.schedule("b");
    const second = scheduler.flush();
    expect(save).toHaveBeenCalledTimes(1); // le 2e attend le 1er
    expect(save).toHaveBeenCalledWith("a");

    resolveFirst!();
    await Promise.all([first, second]);
    expect(save).toHaveBeenCalledTimes(2);
    expect(save).toHaveBeenLastCalledWith("b");
  });
});
