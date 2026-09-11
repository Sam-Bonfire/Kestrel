import { describe, it, expect } from 'vitest';
import { mergeDuplicateEvents } from './duplicates.js';

const ev = (id: string, overrides = {}) => ({
  id,
  title: 'Standup',
  date: '2026-09-08',
  startTime: '09:00',
  endTime: '09:30',
  ...overrides,
});

describe('Duplicate event merger', () => {
  it('merges exact duplicates keeping the first', () => {
    const merged = mergeDuplicateEvents([ev('a'), ev('b'), ev('c')]);
    expect(merged).toHaveLength(1);
    expect(merged[0].id).toBe('a');
    expect(merged[0].duplicateCount).toBe(3);
    expect(merged[0].duplicateIds).toEqual(['b', 'c']);
  });

  it('keeps distinct events separate', () => {
    const merged = mergeDuplicateEvents([ev('a'), ev('b', { startTime: '10:00' }), ev('c', { title: 'Lunch' })]);
    expect(merged).toHaveLength(3);
    expect(merged.every((m) => m.duplicateCount === 1)).toBe(true);
  });

  it('matches case- and whitespace-insensitively', () => {
    const merged = mergeDuplicateEvents([ev('a'), ev('b', { title: '  STANDUP ' })]);
    expect(merged).toHaveLength(1);
  });

  it('does not merge different days', () => {
    const merged = mergeDuplicateEvents([ev('a'), ev('b', { date: '2026-09-09' })]);
    expect(merged).toHaveLength(2);
  });

  it('survives missing fields without throwing', () => {
    const merged = mergeDuplicateEvents([{ id: 'x' }, { id: 'y', title: 'T' }] as any);
    expect(merged).toHaveLength(2);
  });
});
