import { describe, it, expect } from 'vitest';
import { findDuplicateGroups, normalizeContactName } from './merge.js';

describe('Contact merge/deduplication', () => {
  it('normalizes names', () => {
    expect(normalizeContactName('  Jane   DOE! ')).toBe('jane doe');
    expect(normalizeContactName(null)).toBe('');
  });

  it('groups same names with distinct emails', () => {
    const groups = findDuplicateGroups([
      { name: 'Jane Doe', email: 'jane@work.com' },
      { name: 'jane doe', email: 'jane@home.com' },
      { name: 'Bob', email: 'bob@x.com' },
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0].members.map((m) => m.email)).toEqual(['jane@work.com', 'jane@home.com']);
  });

  it('ignores same-email rows and nameless contacts', () => {
    const groups = findDuplicateGroups([
      { name: 'Jane', email: 'j@x.com' },
      { name: 'Jane', email: 'J@X.COM' },
      { name: null, email: 'ghost@x.com' },
    ]);
    expect(groups).toEqual([]);
  });
});
