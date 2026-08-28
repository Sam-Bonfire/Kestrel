import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { triggerUndoAction, executeUndo, dismissToast, activeToasts } from './undoToast.js';
import { get } from 'svelte/store';

describe('undoToast store', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    activeToasts.set([]);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('triggers an action and automatically commits after timeout', async () => {
    const onCommit = vi.fn();
    const onUndo = vi.fn();

    triggerUndoAction({
      title: 'Item deleted',
      timeoutMs: 1000,
      onCommit,
      onUndo,
    });

    expect(get(activeToasts).length).toBe(1);
    expect(get(activeToasts)[0].title).toBe('Item deleted');

    // Advance time past timeout
    await vi.advanceTimersByTimeAsync(1050);

    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onUndo).not.toHaveBeenCalled();
    expect(get(activeToasts).length).toBe(0);
  });

  it('executes undo immediately and prevents commit from running', async () => {
    const onCommit = vi.fn();
    const onUndo = vi.fn();

    const id = triggerUndoAction({
      title: 'Item archived',
      timeoutMs: 3000,
      onCommit,
      onUndo,
    });

    expect(get(activeToasts).length).toBe(1);

    // Call undo
    const success = await executeUndo(id);
    expect(success).toBe(true);
    expect(onUndo).toHaveBeenCalledTimes(1);
    expect(get(activeToasts).length).toBe(0);

    // Advance time past original timeout to ensure commit is cancelled
    await vi.advanceTimersByTimeAsync(3500);
    expect(onCommit).not.toHaveBeenCalled();
  });

  it('executes undo for most recent toast when no ID is specified', async () => {
    const onCommit1 = vi.fn();
    const onUndo1 = vi.fn();
    const onCommit2 = vi.fn();
    const onUndo2 = vi.fn();

    triggerUndoAction({ title: 'Action 1', timeoutMs: 5000, onCommit: onCommit1, onUndo: onUndo1 });
    triggerUndoAction({ title: 'Action 2', timeoutMs: 5000, onCommit: onCommit2, onUndo: onUndo2 });

    expect(get(activeToasts).length).toBe(2);

    await executeUndo(); // Should target Action 2

    expect(onUndo2).toHaveBeenCalledTimes(1);
    expect(onUndo1).not.toHaveBeenCalled();
    expect(get(activeToasts).length).toBe(1);
    expect(get(activeToasts)[0].title).toBe('Action 1');
  });

  it('dismisses toast without commit or undo', async () => {
    const onCommit = vi.fn();
    const onUndo = vi.fn();

    const id = triggerUndoAction({
      title: 'Notification',
      timeoutMs: 5000,
      onCommit,
      onUndo,
    });

    dismissToast(id);
    expect(get(activeToasts).length).toBe(0);

    await vi.advanceTimersByTimeAsync(6000);
    expect(onCommit).not.toHaveBeenCalled();
    expect(onUndo).not.toHaveBeenCalled();
  });
});
