import { describe, it, expect } from 'vitest';
import { findConflicts, nextFreeSlot } from './conflicts.js';

const ev = (id: string, startTime: string, endTime: string, date = '2026-09-09') => ({
  id,
  date,
  startTime,
  endTime,
});

describe('Calendar conflict resolution', () => {
  it('finds overlapping events soonest first', () => {
    const mine = ev('me', '10:00', '11:00');
    const others = [ev('c', '10:30', '11:30'), ev('a', '09:30', '10:15'), ev('far', '14:00', '15:00')];
    expect(findConflicts(mine, others).map((e) => e.id)).toEqual(['a', 'c']);
  });

  it('ignores touching edges, self and other days', () => {
    const mine = ev('me', '10:00', '11:00');
    expect(findConflicts(mine, [ev('me', '10:00', '11:00'), ev('edge', '11:00', '12:00')])).toEqual([]);
    expect(findConflicts(mine, [ev('other', '10:30', '11:30', '2026-09-10')])).toEqual([]);
  });

  it('finds the next free slot after a start time', () => {
    const day = [ev('a', '09:00', '10:00'), ev('b', '10:30', '11:30')];
    expect(nextFreeSlot(day, '2026-09-09', 30, '08:00')).toEqual({ startTime: '08:00', endTime: '08:30' });
    expect(nextFreeSlot(day, '2026-09-09', 60, '09:00')).toEqual({ startTime: '11:30', endTime: '12:30' });
  });

  it('returns null when the day is full', () => {
    const day = [ev('a', '00:00', '23:59')];
    expect(nextFreeSlot(day, '2026-09-09', 30, '08:00')).toBeNull();
  });
});
