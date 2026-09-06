import { writable } from 'svelte/store';

const FOCUS_MODE_KEY = 'kestrel:focus_mode';

function loadInitial(): boolean {
  try {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem(FOCUS_MODE_KEY) === 'true';
    }
  } catch {
    // Non-fatal: fall through to default
  }
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
