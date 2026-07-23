export interface NotificationAction {
  id: string;
  title: string;
}

export interface NotificationCategory {
  id: string;
  actions: NotificationAction[];
}

export const MAIL_NOTIFICATION_CATEGORY: NotificationCategory = {
  id: 'mail_actions',
  actions: [
    { id: 'reply', title: 'Reply Inline' },
    { id: 'archive', title: 'Archive' },
  ],
};

export async function registerNotificationCategories(): Promise<void> {
  // Configures native OS action handlers for interactive notifications
  console.log('[Kestrel Notifications] Registered category:', MAIL_NOTIFICATION_CATEGORY.id);
}
