export interface NotificationAction {
  id: string;
  title: string;
}

export interface NotificationCategory {
  id: string;
  actions: NotificationAction[];
}

export interface DesktopNotification {
  id?: number;
  title: string;
  body?: string;
  actionTypeId?: string;
}

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Ask the OS for notification permission the first time a notification is
 * sent. Returns false outside Tauri or when the user declines.
 */
export async function ensureNotificationPermission(): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    const { isPermissionGranted, requestPermission } = await import(
      '@tauri-apps/plugin-notification'
    );
    if (await isPermissionGranted()) return true;
    return (await requestPermission()) === 'granted';
  } catch {
    return false;
  }
}

/**
 * Register interactive notification actions (inline reply, snooze, ...) with
 * the OS. Must run before sending notifications that reference the category.
 */
export async function registerNotificationCategories(
  categories: NotificationCategory[],
): Promise<void> {
  if (!isTauri()) return;
  try {
    const { registerActionTypes } = await import('@tauri-apps/plugin-notification');
    await registerActionTypes(categories);
  } catch {
    // Browser dev mode or plugin unavailable; notifications are optional.
  }
}

/**
 * Send a desktop notification, requesting permission on first use.
 * Returns false when permission was denied or the app is not running in Tauri.
 */
export async function sendDesktopNotification(
  notification: DesktopNotification,
): Promise<boolean> {
  if (!(await ensureNotificationPermission())) return false;
  try {
    const { sendNotification } = await import('@tauri-apps/plugin-notification');
    sendNotification(notification);
    return true;
  } catch {
    return false;
  }
}
