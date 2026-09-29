export interface FreeBlock {
  /** Minutes since midnight. */
  startMins: number;
  endMins: number;
  mins: number;
}

function toMins(t: string): number {
  const [h = '0', m = '00'] = (t ?? '').split(':');
  const n = Number(h) * 60 + Number(m);
  return Number.isFinite(n) ? Math.max(0, Math.min(24 * 60, n)) : 0;
}

/**
 * Uninterrupted free blocks inside a day, longest first.
 * Pure function over HH:MM busy intervals; overlaps merge.
 */
export function computeFreeBlocks(
  busy: { startTime?: string | null; endTime?: string | null }[],
  minMins = 120,
  dayEnd = 24 * 60,
): FreeBlock[] {
  const spans = busy
    .map((b) => {
      const s = toMins(b.startTime ?? '00:00');
      let e = toMins(b.endTime ?? '00:00');
      // Overnight spans (end before start) busy the rest of this day.
      // Multi-day rendering clips per day, same as event chips.
      if (e < s) e = dayEnd;
      e = Math.max(s, Math.min(dayEnd, e));
      return [s, e] as const;
    })
    .sort((a, b) => a[0] - b[0]);
  const merged: [number, number][] = [];
  for (const [s, e] of spans) {
    const last = merged[merged.length - 1];
    if (last && s <= last[1]) last[1] = Math.max(last[1], e);
    else merged.push([s, e]);
  }
  const out: FreeBlock[] = [];
  let cursor = 0;
  for (const [s, e] of merged) {
    if (s - cursor >= minMins) {
      out.push({ startMins: cursor, endMins: s, mins: s - cursor });
    }
    cursor = Math.max(cursor, e);
  }
  if (dayEnd - cursor >= minMins) {
    out.push({ startMins: cursor, endMins: dayEnd, mins: dayEnd - cursor });
  }
  return out.sort((a, b) => b.mins - a.mins);
}

export function formatFreeBlock(mins: number): string {
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  if (h === 0) return `${m}m free`;
  return m === 0 ? `${h}h free` : `${h}h ${m}m free`;
}
