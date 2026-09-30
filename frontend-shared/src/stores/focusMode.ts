import { writable } from 'svelte/store';

const FOCUS_MODE_KEY = 'kestrel:focus_mode';

function loadInitial(): boolean {
  // Always start off: Focus/batch queues are session-only, so a reload
  // mid-batch must not resurrect the collapsed UI without its queue.
  // The persisted write below is kept for future subscribers.
  return false;
}

export const focusMode = writable<boolean>(loadInitial());

focusMode.subscribe((val) => {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(FOCUS_MODE_KEY, String(val));
    }
  } catch {
    // Non-fatal
  }
});

export function toggleFocusMode() {
  focusMode.update((v) => !v);
}

export function exitFocusMode() {
  focusMode.set(false);
}
