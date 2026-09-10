import { describe, it, expect } from 'vitest';
import { resolveEffectiveTheme, applyTheme } from './theme.js';

describe('Theme engine', () => {
  it('resolves explicit modes regardless of OS', () => {
    expect(resolveEffectiveTheme('dark', false)).toBe('dark');
    expect(resolveEffectiveTheme('light', true)).toBe('light');
  });

  it('follows the OS in system mode', () => {
    expect(resolveEffectiveTheme('system', true)).toBe('dark');
    expect(resolveEffectiveTheme('system', false)).toBe('light');
  });

  it('falls back to the OS signal for unknown modes', () => {
    expect(resolveEffectiveTheme('bogus', true)).toBe('dark');
    expect(resolveEffectiveTheme('bogus', false)).toBe('light');
  });

  it('applies the effective theme to the document root', () => {
    applyTheme('light');
    expect(document.documentElement.dataset.theme).toBe('light');
    expect(document.documentElement.style.colorScheme).toBe('light');
    applyTheme('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
  });
});
