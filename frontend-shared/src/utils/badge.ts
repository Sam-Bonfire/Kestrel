type BadgeNavigator = Navigator & {
  setAppBadge?: (count: number) => Promise<void>;
  clearAppBadge?: () => Promise<void>;
};

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Mirror the unread count onto the OS dock/taskbar badge (Tauri's
 * built-in Badging API — no plugin) or the PWA badging API in browsers.
 * Zero/negative clears the badge. Never throws.
 */
export async function setUnreadBadge(count: number): Promise<void> {
  const badge = count > 0 ? count : undefined;
  try {
    if (isTauri()) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().setBadgeCount(badge);
      return;
    }
    const nav: BadgeNavigator | null =
      typeof navigator !== 'undefined' ? (navigator as BadgeNavigator) : null;
    if (nav && typeof nav.setAppBadge === 'function') {
      if (badge === undefined && typeof nav.clearAppBadge === 'function') {
        await nav.clearAppBadge();
      } else if (badge !== undefined) {
        await nav.setAppBadge(badge);
      }
    }
  } catch {
    // Badges are best-effort; the in-app counts remain the source of truth.
  }
}
