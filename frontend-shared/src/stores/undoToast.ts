import { writable, get } from 'svelte/store';

export interface UndoAction {
  id: string;
  title: string;
  description?: string;
  timeoutMs: number;
  remainingMs: number;
  onCommit: () => Promise<void> | void;
  onUndo: () => Promise<void> | void;
  type?: 'info' | 'success' | 'warning' | 'error';
}

interface ToastInternal extends UndoAction {
  intervalId?: ReturnType<typeof setInterval>;
  timeoutId?: ReturnType<typeof setTimeout>;
}

export const activeToasts = writable<ToastInternal[]>([]);

/**
 * Trigger an undoable action.
 * Shows a toast with a countdown timer (default 5s).
 * If Undo is clicked (or 'Z' is pressed), onUndo is called immediately and onCommit is never called.
 * If the timer expires, onCommit is called and the toast is dismissed.
 */
export function triggerUndoAction(options: {
  title: string;
  description?: string;
  timeoutMs?: number;
  onCommit: () => Promise<void> | void;
  onUndo: () => Promise<void> | void;
  type?: 'info' | 'success' | 'warning' | 'error';
}): string {
  const id = `undo-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
  const timeoutMs = options.timeoutMs ?? 5000;
  const stepMs = 50;

  const toast: ToastInternal = {
    id,
    title: options.title,
    description: options.description,
    timeoutMs,
    remainingMs: timeoutMs,
    onCommit: options.onCommit,
    onUndo: options.onUndo,
    type: options.type ?? 'info',
  };

  const intervalId = setInterval(() => {
    activeToasts.update(list => {
      return list.map(t => {
        if (t.id === id) {
          const nextRemaining = Math.max(0, t.remainingMs - stepMs);
          return { ...t, remainingMs: nextRemaining };
        }
        return t;
      });
    });
  }, stepMs);

  const timeoutId = setTimeout(async () => {
    cleanupToastTimers(id);
    const current = get(activeToasts).find(t => t.id === id);
    activeToasts.update(list => list.filter(t => t.id !== id));
    if (current) {
      try {
        await current.onCommit();
      } catch (err) {
        console.error(`Error committing action ${current.title}:`, err);
      }
    }
  }, timeoutMs);

  toast.intervalId = intervalId;
  toast.timeoutId = timeoutId;

  activeToasts.update(list => [...list, toast]);
  return id;
}

function cleanupToastTimers(id: string) {
  const current = get(activeToasts).find(t => t.id === id);
  if (current) {
    if (current.intervalId) clearInterval(current.intervalId);
    if (current.timeoutId) clearTimeout(current.timeoutId);
  }
}

/**
 * Execute undo for a specific toast ID (or most recent if not specified).
 */
export async function executeUndo(id?: string): Promise<boolean> {
  const list = get(activeToasts);
  if (list.length === 0) return false;

  const targetToast = id ? list.find(t => t.id === id) : list[list.length - 1];
  if (!targetToast) return false;

  cleanupToastTimers(targetToast.id);
  activeToasts.update(currentList => currentList.filter(t => t.id !== targetToast.id));

  try {
    await targetToast.onUndo();
    return true;
  } catch (err) {
    console.error(`Error undoing action ${targetToast.title}:`, err);
    return false;
  }
}

/**
 * Dismiss a toast without executing commit or undo (e.g. forced cancel).
 */
export function dismissToast(id: string) {
  cleanupToastTimers(id);
  activeToasts.update(list => list.filter(t => t.id !== id));
}
