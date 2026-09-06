export interface DetectedTimezoneResult {
  originalText: string;
  sourceTime: string;
  sourceTimezone: string;
  localDate: Date;
  formattedLocalTime: string;
}

const TIMEZONE_MAP: Record<string, string> = {
  EST: "America/New_York",
  EDT: "America/New_York",
  CST: "America/Chicago",
  CDT: "America/Chicago",
  MST: "America/Denver",
  MDT: "America/Denver",
  PST: "America/Los_Angeles",
  PDT: "America/Los_Angeles",
  UTC: "UTC",
  GMT: "Europe/London",
  BST: "Europe/London",
  CET: "Europe/Paris",
  CEST: "Europe/Paris",
  IST: "Asia/Kolkata",
  JST: "Asia/Tokyo",
  AEST: "Australia/Sydney",
  AEDT: "Australia/Sydney"
};

const TIME_REGEX = /\b((?:1[0-2]|0?[1-9])(?::[0-5][0-9])?\s*(?:am|pm|AM|PM)|(?:[01]?[0-9]|2[0-3]):[0-5][0-9])\s*([A-Z]{3,4})\b/i;

export function detectTimezone(text: string): DetectedTimezoneResult | null {
  if (!text) return null;

  const match = text.match(TIME_REGEX);
  if (!match) return null;

  const fullMatch = match[0];
  const timePart = match[1];
  const tzPart = match[2].toUpperCase();

  const ianaTz = TIMEZONE_MAP[tzPart];
  if (!ianaTz) return null;

  const userLocalTz = Intl.DateTimeFormat().resolvedOptions().timeZone;

  const now = new Date();
  let hours = 0;
  let minutes = 0;

  const is12Hour = /am|pm/i.test(timePart);
  if (is12Hour) {
    const timeRegex12 = /^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$/i;
    const tMatch = timePart.match(timeRegex12);
    if (tMatch) {
      let h = parseInt(tMatch[1], 10);
      const m = tMatch[2] ? parseInt(tMatch[2], 10) : 0;
      const ampm = tMatch[3].toLowerCase();

      if (ampm === "pm" && h < 12) h += 12;
      if (ampm === "am" && h === 12) h = 0;

      hours = h;
      minutes = m;
    }
  } else {
    const [h, m] = timePart.split(":");
    hours = parseInt(h, 10);
    minutes = parseInt(m, 10);
  }

  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');

  const getOffset = (date: Date, timeZone: string) => {
    const tzDate = new Date(date.toLocaleString('en-US', { timeZone }));
    const utcDate = new Date(date.toLocaleString('en-US', { timeZone: 'UTC' }));
    return (tzDate.getTime() - utcDate.getTime()) / 60000;
  };

  const sourceOffset = getOffset(now, ianaTz);
  const localOffset = getOffset(now, userLocalTz);

  const utcDate = new Date(Date.UTC(year, now.getMonth(), now.getDate(), hours, minutes));
  utcDate.setMinutes(utcDate.getMinutes() - sourceOffset);

  const localDate = utcDate;

  const formatter = new Intl.DateTimeFormat('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  });

  const formattedLocalTime = formatter.format(localDate);

  if (sourceOffset === localOffset) {
    return null;
  }

  return {
    originalText: fullMatch,
    sourceTime: timePart,
    sourceTimezone: tzPart,
    localDate: localDate,
    formattedLocalTime
  };
}
