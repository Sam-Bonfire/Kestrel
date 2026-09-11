export interface MergeableContact {
  name?: string | null;
  email: string;
}

export interface DuplicateGroup {
  /** Normalized display name shared by the group */
  key: string;
  members: MergeableContact[];
}

/** Normalize a display name for grouping (case, punctuation, spacing). */
export function normalizeContactName(name: string | null | undefined): string {
  return (name ?? '')
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Group contacts that likely describe the same person: same normalized
 * name with more than one distinct email. Nameless rows never group.
 *
 * ponytail: exact normalized-name match only, no fuzzy similarity.
 * Fuzzy merging needs per-row user confirmation beyond keep/delete.
 */
export function findDuplicateGroups<T extends MergeableContact>(contacts: T[]): DuplicateGroup[] {
  const byKey = new Map<string, T[]>();
  for (const c of contacts) {
    const key = normalizeContactName(c.name);
    if (!key) continue;
    const list = byKey.get(key) ?? [];
    list.push(c);
    byKey.set(key, list);
  }
  return [...byKey.entries()]
    .filter(([, members]) => new Set(members.map((m) => m.email.toLowerCase())).size > 1)
    .map(([key, members]) => ({ key, members: [...members] }))
    .sort((a, b) => b.members.length - a.members.length || a.key.localeCompare(b.key));
}
