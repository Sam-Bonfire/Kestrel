import { afterEach, describe, expect, it, vi } from 'vitest';
import { loadTurnstile, type TurnstileApi } from './turnstile.js';

const api: TurnstileApi = { render: vi.fn(() => 'widget-1'), remove: vi.fn() };

afterEach(() => {
  delete window.turnstile;
  document.head.querySelectorAll('script[src*="turnstile"]').forEach((el) => el.remove());
});

describe('loadTurnstile', () => {
  it('loads the script once and reuses the same API', async () => {
    const first = loadTurnstile();
    const script = document.head.querySelector<HTMLScriptElement>('script[src*="turnstile"]');
    expect(script).not.toBeNull();
    expect(loadTurnstile()).toBe(first);

    window.turnstile = api;
    script?.onload?.(new Event('load'));

    await expect(first).resolves.toBe(api);
    expect(document.head.querySelectorAll('script[src*="turnstile"]').length).toBe(1);
    await expect(loadTurnstile()).resolves.toBe(api);
    expect(document.head.querySelectorAll('script[src*="turnstile"]').length).toBe(1);
  });

  it('rejects when the script cannot load so the form can surface it', async () => {
    vi.resetModules();
    const fresh = await import('./turnstile.js');

    const pending = fresh.loadTurnstile();
    const script = document.head.querySelector<HTMLScriptElement>('script[src*="turnstile"]');
    script?.onerror?.(new Event('error'));

    await expect(pending).rejects.toThrow('Could not load Turnstile');
  });
});
