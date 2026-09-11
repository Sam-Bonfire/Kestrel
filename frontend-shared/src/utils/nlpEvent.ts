export interface ParsedEvent {
  title: string;
  /** Local YYYY-MM-DD */
  date: string;
  /** HH:MM 24h */
  startTime: string;
  durationMins: number;
}

/** Add minutes to an HH:MM time, reporting how many days forward it lands. */
export function shiftTime(time: string, mins: number): { endTime: string; daysAfter: number } {
  const [h, m] = time.split(':').map(Number);
  const total = h * 60 + m + mins;
  const wrapped = ((total % (24 * 60)) + 24 * 60) % (24 * 60);
  return {
    endTime: `${String(Math.floor(wrapped / 60)).padStart(2, '0')}:${String(wrapped % 60).padStart(2, '0')}`,
    daysAfter: Math.floor(total / (24 * 60)),
  };
}

const WEEKDAYS = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];

function toISODate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

function addDays(base: Date, n: number): Date {
  const d = new Date(base);
  d.setDate(d.getDate() + n);
  return d;
}

/**
 * Deterministic natural-language event parser ("lunch tomorrow 12:30 1h").
 * Handles relative days, weekday names, times and durations; the
 * remainder is the title.
 *
 * ponytail: regex subset, no date NLP library. Extend patterns here
 * when real inputs fail; reach for a library only if this file
 * grows past ~150 lines.
 */
export function parseNaturalEvent(input: string, now: Date = new Date()): ParsedEvent | null {
  const text = input.trim();
  if (!text) return null;
  let rest = ` ${text.toLowerCase()} `;
  let date = new Date(now);

  const take = (pattern: RegExp): string | null => {
    const m = rest.match(pattern);
    if (!m) return null;
    rest = rest.replace(m[0], ' ');
    return m[0];
  };

  if (take(/\bday after tomorrow\b/)) {
    date = addDays(now, 2);
  } else if (take(/\btomorrow\b/)) {
    date = addDays(now, 1);
  } else if (take(/\btoday\b/) || take(/\btonight\b/)) {
    date = new Date(now);
  } else {
    const nextMatch = rest.match(/\bnext\s+(sunday|monday|tuesday|wednesday|thursday|friday|saturday)\b/);
    const dayMatch = nextMatch ?? rest.match(/\b(sunday|monday|tuesday|wednesday|thursday|friday|saturday)\b/);
    if (dayMatch) {
      rest = rest.replace(dayMatch[0], ' ');
      const target = WEEKDAYS.indexOf(dayMatch[1]);
      let delta = (target - now.getDay() + 7) % 7;
      if (delta === 0 || nextMatch) delta += 7;
      date = addDays(now, delta);
    } else {
      const iso = rest.match(/\b(\d{4})-(\d{2})-(\d{2})\b/);
      if (iso) {
        rest = rest.replace(iso[0], ' ');
        date = new Date(Number(iso[1]), Number(iso[2]) - 1, Number(iso[3]));
      }
    }
  }

  // Duration first so "1h" isn't eaten as a time. Sums compounds ("1h30m"),
  // clamped to 1 minute .. 1 day. Skipped after a bare "in" ("meeting in
  // 2 hours" is a title, not a duration).
  let durationMins = 60;
  if (!/\bin\s+\d+\s*(hours?|hrs?|minutes?|mins?|m|h|days?|weeks?)\b/.test(rest)) {
    rest = rest.replace(/(\d+\s*h)\s*(\d)/gi, '$1 $2'); // split "1h30m"
    const found = [...rest.matchAll(/(\d+)\s*(hours?|hrs?|minutes?|mins?|m|h)\b/gi)];
    if (found.length > 0) {
      for (const m of found) rest = rest.replace(m[0], ' ');
      const total = found.reduce(
        (sum, m) => sum + (m[2].toLowerCase().startsWith('h') ? Number(m[1]) * 60 : Number(m[1])),
        0
      );
      durationMins = Math.min(1440, Math.max(1, total));
    }
  }

  // Times: "at 3pm", "15:00", "9:30am", ranges "2-3pm" (takes start).
  let startTime = '09:00';
  const range =
    rest.match(/(?:\bat\s+)?(\d{1,2})(?::(\d{2}))?\s*[-–]\s*(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b/) ??
    rest.match(/(?:\bat\s+)?(\d{1,2})(?::(\d{2}))?\s*[-–]\s*(\d{1,2})(?::(\d{2}))?\b/);
  const single =
    rest.match(/\bat\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm)?\b/) ??
    rest.match(/\b(\d{1,2}):(\d{2})\s*(am|pm)?\b/) ??
    rest.match(/\b(\d{1,2})\s*(am|pm)\b/);
  const timeMatch = range ?? single;
  if (timeMatch) {
    const raw = timeMatch[0];
    let hours = Number((raw.match(/\d{1,2}/) as RegExpMatchArray)[0]);
    const minMatch = raw.match(/:(\d{2})/);
    const minutes = minMatch ? Number(minMatch[1]) : 0;
    const meridiem = /pm\b/.test(raw) ? 'pm' : /am\b/.test(raw) ? 'am' : undefined;
    if (meridiem === 'pm' && hours < 12) hours += 12;
    if (meridiem === 'am' && hours === 12) hours = 0;
    if (hours <= 23 && minutes <= 59) {
      rest = rest.replace(raw, ' ');
      startTime = `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
    }
    // Invalid times stay in the title; the 09:00 default stands.
  }

  const title = rest.replace(/\s+/g, ' ').trim() || 'Untitled event';
  return {
    title: title.charAt(0).toUpperCase() + title.slice(1),
    date: toISODate(date),
    startTime,
    durationMins,
  };
}
