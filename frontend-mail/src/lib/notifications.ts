export interface NotificationAction {
  id: string;
  title: string;
}

export interface NotificationCategory {
  id: string;
  actions: NotificationAction[];
}

/**
 * Single source of truth for the Mail app's interactive notification category.
 * Registered with the OS at startup via registerNotificationCategories().
 */
export const MAIL_NOTIFICATION_CATEGORY: NotificationCategory = {
  id: 'new_email_actions',
  actions: [
    { id: 'reply', title: 'Reply Inline' },
    { id: 'mark_read', title: 'Mark as Read' },
    { id: 'archive', title: 'Archive' },
  ],
};

/**
 * Registers the interactive action category so that OS notifications
 * surface inline Reply, Mark as Read, and Archive actions.
 * Safe to call in any environment — no-ops outside Tauri.
 */
export async function registerNotificationCategories(): Promise<void> {
  try {
    const { registerActionTypes } = await import('@tauri-apps/plugin-notification');
    await registerActionTypes([MAIL_NOTIFICATION_CATEGORY]);
  } catch {
    // Not running inside Tauri — ignore (browser dev mode).
  }
}
