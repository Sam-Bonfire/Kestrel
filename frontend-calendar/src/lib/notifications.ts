export interface CalendarNotificationCategory {
  id: string;
  actions: { id: string; title: string }[];
}

export const CALENDAR_NOTIFICATION_CATEGORY: CalendarNotificationCategory = {
  id: 'calendar_event_reminder',
  actions: [
    { id: 'snooze_15m', title: 'Snooze 15m' },
    { id: 'join_meeting', title: 'Join Meeting Link' },
  ],
};

export async function registerCalendarNotifications(): Promise<void> {
  console.log('[Kestrel Calendar] Registered notification category:', CALENDAR_NOTIFICATION_CATEGORY.id);
}
