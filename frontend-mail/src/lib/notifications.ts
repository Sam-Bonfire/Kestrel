import {
  registerNotificationCategories as registerCategories,
  sendDesktopNotification,
  type NotificationCategory,
} from '@kestrel/shared';

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
 * Safe to call in any environment - no-ops outside Tauri.
 */
export function registerNotificationCategories(): Promise<void> {
  return registerCategories([MAIL_NOTIFICATION_CATEGORY]);
}

/**
 * Sends a mail notification, asking for OS permission on first use.
 * Resolves false in browser dev mode or when permission is denied.
 */
export function sendMailNotification(notification: {
  title: string;
  body?: string;
  actionTypeId?: string;
}): Promise<boolean> {
  return sendDesktopNotification(notification);
}
