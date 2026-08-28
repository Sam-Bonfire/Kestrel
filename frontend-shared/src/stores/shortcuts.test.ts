import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  inputGuard,
  customShortcuts,
  updateShortcut,
  resetShortcuts,
  defaultShortcuts,
} from './shortcuts.js';
import { get } from 'svelte/store';

// In-memory localStorage mock for node test environment
const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Shortcuts Engine & Input Guard', () => {
  beforeEach(() => {
    resetShortcuts();
    localStorage.clear();
  });

  describe('inputGuard', () => {
    it('suppresses single-key shortcuts when typing in an INPUT element', () => {
      const target = { tagName: 'INPUT', isContentEditable: false } as unknown as HTMLElement;
      const event = { key: 'c', target, ctrlKey: false, metaKey: false } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(true);
    });

    it('suppresses single-key shortcuts when typing in a TEXTAREA', () => {
      const target = { tagName: 'TEXTAREA', isContentEditable: false } as unknown as HTMLElement;
      const event = { key: 'e', target, ctrlKey: false, metaKey: false } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(true);
    });

    it('suppresses single-key shortcuts inside contenteditable elements', () => {
      const target = { tagName: 'DIV', isContentEditable: true } as unknown as HTMLElement;
      const event = { key: 'i', target, ctrlKey: false, metaKey: false } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(true);
    });

    it('allows modifier chords (Cmd/Ctrl+K) to bypass the input guard', () => {
      const target = { tagName: 'INPUT', isContentEditable: false } as unknown as HTMLElement;
      const event = { key: 'k', ctrlKey: true, metaKey: false, target } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(false);
    });

    it('allows Escape key to bypass the input guard', () => {
      const target = { tagName: 'INPUT', isContentEditable: false } as unknown as HTMLElement;
      const event = { key: 'Escape', ctrlKey: false, metaKey: false, target } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(false);
    });

    it('allows single-key shortcuts when focus is on body/div container', () => {
      const target = { tagName: 'DIV', isContentEditable: false } as unknown as HTMLElement;
      const event = { key: 'c', ctrlKey: false, metaKey: false, target } as unknown as KeyboardEvent;

      expect(inputGuard(event)).toBe(false);
    });
  });

  describe('customShortcuts store', () => {
    it('updates and persists shortcut overrides', () => {
      updateShortcut('compose', 'N');
      const current = get(customShortcuts);
      expect(current['compose']).toBe('N');
    });

    it('resets custom shortcuts to empty overrides', () => {
      updateShortcut('archive', 'Y');
      resetShortcuts();
      const current = get(customShortcuts);
      expect(current).toEqual({});
    });

    it('contains all default shortcut definitions', () => {
      expect(defaultShortcuts.length).toBeGreaterThan(5);
      const commandShortcut = defaultShortcuts.find(s => s.id === 'command');
      expect(commandShortcut?.defaultKey).toBe('$mod+K');
    });
  });
});
