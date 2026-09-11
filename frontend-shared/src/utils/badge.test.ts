import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setUnreadBadge } from './badge.js';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    setBadgeCount: (...args: unknown[]) => (globalThis as any).__mockSetBadgeCount?.(...args),
  }),
}));

type BadgeNavigator = Navigator & {
  setAppBadge?: (count: number) => Promise<void>;
  clearAppBadge?: () => Promise<void>;
};

describe('Unread badge', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    delete (window as any).__TAURI_INTERNALS__;
    delete (navigator as unknown as BadgeNavigator).setAppBadge;
    delete (navigator as unknown as BadgeNavigator).clearAppBadge;
    delete (globalThis as any).__mockSetBadgeCount;
  });

  it('resolves silently without any badge API', async () => {
    await expect(setUnreadBadge(5)).resolves.toBeUndefined();
    await expect(setUnreadBadge(0)).resolves.toBeUndefined();
  });

  it('uses the PWA badging API when present', async () => {
    const setAppBadge = vi.fn().mockResolvedValue(undefined);
    const clearAppBadge = vi.fn().mockResolvedValue(undefined);
    (navigator as unknown as BadgeNavigator).setAppBadge = setAppBadge;
    (navigator as unknown as BadgeNavigator).clearAppBadge = clearAppBadge;

    await setUnreadBadge(3);
    expect(setAppBadge).toHaveBeenCalledWith(3);
    await setUnreadBadge(0);
    expect(clearAppBadge).toHaveBeenCalled();
  });

  it('uses the Tauri badge count and clears on zero', async () => {
    const setBadgeCount = vi.fn().mockResolvedValue(undefined);
    (globalThis as any).__mockSetBadgeCount = setBadgeCount;
    (window as any).__TAURI_INTERNALS__ = {};

    await setUnreadBadge(7);
    expect(setBadgeCount).toHaveBeenCalledWith(7);
    await setUnreadBadge(0);
    expect(setBadgeCount).toHaveBeenCalledWith(undefined);
    await setUnreadBadge(-2);
    expect(setBadgeCount).toHaveBeenCalledWith(undefined);
  });

  it('swallows backend failures', async () => {
    (globalThis as any).__mockSetBadgeCount = vi.fn().mockRejectedValue(new Error('denied'));
    (window as any).__TAURI_INTERNALS__ = {};
    await expect(setUnreadBadge(4)).resolves.toBeUndefined();
  });
});
