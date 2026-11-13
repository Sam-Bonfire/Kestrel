import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

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

  it('always starts off even when a value is persisted', async () => {
    // Batch queues are session-only: a reload must not resurrect focus UI.
    localStorage.setItem('kestrel:focus_mode', 'true');
    const { focusMode } = await import('./focusMode.js');
    expect(get(focusMode)).toBe(false);
  });
});
