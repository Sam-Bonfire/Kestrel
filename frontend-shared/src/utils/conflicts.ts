export interface Schedulable {
  id: string;
  date: string;
  startTime: string;
  endTime: string;
  title?: string;
}

function toMins(time: string): number {
  const [h = '0', m = '00'] = (time ?? '').split(':');
  return Number(h) * 60 + Number(m);
}

function toTime(mins: number): string {
  const clamped = Math.max(0, Math.min(24 * 60 - 1, mins));
  return `${String(Math.floor(clamped / 60)).padStart(2, '0')}:${String(clamped % 60).padStart(2, '0')}`;
}

function overlaps(a: Schedulable, b: Schedulable): boolean {
  if (a.date !== b.date || a.id === b.id) return false;
  return toMins(a.startTime) < toMins(b.endTime) && toMins(b.startTime) < toMins(a.endTime);
}

/**
 * Events overlapping the given one, soonest first.
 *
 * ponytail: same-day time overlap only, no travel buffers or priority
 * arbitration. Smarter resolution (attendee ranks, working hours)
 * belongs in a follow-up once this ships.
 */
export function findConflicts(event: Schedulable, others: Schedulable[]): Schedulable[] {
  return others
    .filter((o) => overlaps(event, o))
    .sort((a, b) => a.startTime.localeCompare(b.startTime));
}

export interface FreeSlot {
  startTime: string;
  endTime: string;
}

/**
 * Next free slot on the same day fitting durationMins at or after fromTime.
 * Scans gaps between existing events; null when the day is full.
 */
export function nextFreeSlot(
  events: Schedulable[],
  date: string,
  durationMins: number,
  fromTime = '08:00'
): FreeSlot | null {
  const day = events
    .filter((e) => e.date === date)
    .map((e) => ({ start: toMins(e.startTime), end: toMins(e.endTime) }))
    .sort((a, b) => a.start - b.start);
  let cursor = toMins(fromTime);
  const dayEnd = 24 * 60 - 1;
  for (const e of day) {
    if (e.start - cursor >= durationMins) {
      return { startTime: toTime(cursor), endTime: toTime(cursor + durationMins) };
    }
    cursor = Math.max(cursor, e.end);
  }
  if (dayEnd - cursor >= durationMins) {
    return { startTime: toTime(cursor), endTime: toTime(cursor + durationMins) };
  }
  return null;
}
