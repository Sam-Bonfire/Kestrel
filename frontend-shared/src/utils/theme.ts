import { get } from 'svelte/store';
import { theme } from '../stores/settings.js';

export type ThemeMode = 'dark' | 'light' | 'system';

/** Resolve the effective theme, following the OS when in system mode. */
export function resolveEffectiveTheme(mode: string, prefersDark: boolean): 'dark' | 'light' {
  if (mode === 'light') return 'light';
  if (mode === 'dark') return 'dark';
  return prefersDark ? 'dark' : 'light';
}

function systemPrefersDark(): boolean {
  try {
    return typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: dark)').matches;
  } catch {
    return true;
  }
}

/** Apply a theme mode to the document root. */
export function applyTheme(mode: string) {
  if (typeof document === 'undefined') return;
  const effective = resolveEffectiveTheme(mode, systemPrefersDark());
  document.documentElement.dataset.theme = effective;
  document.documentElement.style.colorScheme = effective;
}

/**
 * Apply the stored theme and keep it in sync with the OS while in
 * system mode. Call once per app root; returns a cleanup function.
 */
export function initTheme(): () => void {
  applyTheme(get(theme));
  const unsubscribe = theme.subscribe((mode) => applyTheme(mode));
  let media: MediaQueryList | null = null;
  const onChange = () => applyTheme(get(theme));
  try {
    if (typeof matchMedia !== 'undefined') {
      media = matchMedia('(prefers-color-scheme: dark)');
      media.addEventListener('change', onChange);
    }
  } catch {
    media = null;
  }
  return () => {
    unsubscribe();
    try {
      media?.removeEventListener('change', onChange);
    } catch {
      // Non-fatal
    }
  };
}
