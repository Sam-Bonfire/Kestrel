export interface BriefingEvent {
  title: string;
  startTime: string;
  location?: string;
}

export interface BriefingInput {
  events: BriefingEvent[];
  unreadCount: number;
}

export interface DailyBriefing {
  title: string;
  body: string;
  eventCount: number;
  unreadCount: number;
}

const SHOWN_KEY = 'kestrel:briefing:last-shown';

function toISODate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

/** True once per calendar day (local time). */
export function shouldShowBriefing(now: Date = new Date()): boolean {
  try {
    if (typeof localStorage === 'undefined') return false;
    return localStorage.getItem(SHOWN_KEY) !== toISODate(now);
  } catch {
    return false;
  }
}

export function markBriefingShown(now: Date = new Date()) {
  try {
    if (typeof localStorage !== 'undefined') localStorage.setItem(SHOWN_KEY, toISODate(now));
  } catch {
    // Non-fatal
  }
}

/**
 * Compose a morning briefing from today's agenda and inbox pressure.
 * Pure function; callers supply data and deliver via OS notification.
 */
export function buildDailyBriefing(input: BriefingInput): DailyBriefing {
  const timed = [...input.events].sort((a, b) => a.startTime.localeCompare(b.startTime));
  const lines = timed.slice(0, 5).map((e) => `• ${e.startTime} ${e.title}`);
  if (timed.length > 5) lines.push(`• +${timed.length - 5} more`);
  if (input.unreadCount > 0) lines.push(`• ${input.unreadCount} unread message${input.unreadCount === 1 ? '' : 's'}`);
  if (lines.length === 0) lines.push('• Nothing scheduled. Inbox clear. Enjoy the calm.');
  const first = timed[0];
  return {
    title: first ? `Today: ${timed.length} event${timed.length === 1 ? '' : 's'}, first at ${first.startTime}` : 'Today at a glance',
    body: lines.join('\n'),
    eventCount: timed.length,
    unreadCount: input.unreadCount,
  };
}
