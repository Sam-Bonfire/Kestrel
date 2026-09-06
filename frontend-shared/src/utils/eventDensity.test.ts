import { describe, it, expect } from 'vitest';
import { buildEventDensityMap, daysInMonth, toISODateString } from './eventDensity.js';

describe('eventDensity', () => {
  it('aggregates counts and caps titles at 3', () => {
    const map = buildEventDensityMap([
      { date: '2026-09-07', title: 'A' },
      { date: '2026-09-07', title: 'B' },
      { date: '2026-09-07', title: 'C' },
      { date: '2026-09-07', title: 'D' },
      { date: '2026-09-08', title: 'E' },
    ]);
    expect(map['2026-09-07']).toEqual({ count: 4, titles: ['A', 'B', 'C'] });
    expect(map['2026-09-08']).toEqual({ count: 1, titles: ['E'] });
    expect(map['2026-09-09']).toBeUndefined();
  });

  it('handles leap years and month lengths', () => {
    expect(daysInMonth(2024, 1)).toBe(29);
    expect(daysInMonth(2026, 1)).toBe(28);
    expect(daysInMonth(2026, 0)).toBe(31);
    expect(toISODateString(2026, 0, 5)).toBe('2026-01-05');
  });
});
