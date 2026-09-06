export interface DatedEvent {
  /** YYYY-MM-DD */
  date: string;
  title: string;
}

export interface DayDensity {
  count: number;
  /** Up to 3 titles, in event order */
  titles: string[];
}

/** Aggregate events into a YYYY-MM-DD -> density map for heatmap rendering. */
export function buildEventDensityMap(events: DatedEvent[]): Record<string, DayDensity> {
  const map: Record<string, DayDensity> = {};
  for (const ev of events) {
    const entry = map[ev.date];
    if (entry) {
      entry.count += 1;
      if (entry.titles.length < 3) entry.titles.push(ev.title);
    } else {
      map[ev.date] = { count: 1, titles: [ev.title] };
    }
  }
  return map;
}

export function toISODateString(year: number, month: number, day: number): string {
  return `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

/** Days in a month, leap-year aware via the Date API. */
export function daysInMonth(year: number, month: number): number {
  return new Date(year, month + 1, 0).getDate();
}
