export interface MergeableEvent {
  id: string;
  title: string;
  date: string;
  startTime: string;
  endTime: string;
}

export type MergedEvent<T extends MergeableEvent> = T & {
  duplicateCount: number;
  duplicateIds: string[];
};

function normalizeTime(t: string | null | undefined): string {
  const [h = '0', m = '00'] = (t ?? '').split(':');
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
}

function mergeKey(e: MergeableEvent): string {
  return [(e.title ?? '').trim().toLowerCase(), e.date ?? '', normalizeTime(e.startTime), normalizeTime(e.endTime)].join('|');
}

/**
 * Collapse exact duplicates (same title + slot, e.g. synced from two
 * providers) into one chip, keeping the first occurrence.
 *
 * ponytail: exact-match only, no fuzzy similarity. Fuzzy dedupe belongs
 * in a contact-merge style tool with user confirmation, not silent merge.
 */
export function mergeDuplicateEvents<T extends MergeableEvent>(events: T[]): MergedEvent<T>[] {
  const byKey = new Map<string, MergedEvent<T>>();
  for (const e of events) {
    const existing = byKey.get(mergeKey(e));
    if (existing) {
      existing.duplicateCount += 1;
      existing.duplicateIds.push(e.id);
    } else {
      byKey.set(mergeKey(e), { ...e, duplicateCount: 1, duplicateIds: [] });
    }
  }
  return [...byKey.values()];
}
