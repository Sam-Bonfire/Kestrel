import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

// In-memory localStorage mock for node test environment
const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach((k) => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('focusMode store', () => {
  beforeEach(async () => {
    localStorage.clear();
    vi.resetModules();
  });

  it('defaults to false when nothing is persisted', async () => {
    const { focusMode } = await import('./focusMode.js');
    expect(get(focusMode)).toBe(false);
  });

  it('toggles the state and persists it', async () => {
    const { focusMode, toggleFocusMode } = await import('./focusMode.js');
    toggleFocusMode();
    expect(get(focusMode)).toBe(true);
    expect(localStorage.getItem('kestrel:focus_mode')).toBe('true');
    toggleFocusMode();
    expect(get(focusMode)).toBe(false);
  });

  it('exits focus mode explicitly', async () => {
    const { focusMode, toggleFocusMode, exitFocusMode } = await import('./focusMode.js');
    toggleFocusMode();
    exitFocusMode();
    expect(get(focusMode)).toBe(false);
    expect(localStorage.getItem('kestrel:focus_mode')).toBe('false');
  });

  it('initializes from persisted value', async () => {
    localStorage.setItem('kestrel:focus_mode', 'true');
    const { focusMode } = await import('./focusMode.js');
    expect(get(focusMode)).toBe(true);
  });
});
