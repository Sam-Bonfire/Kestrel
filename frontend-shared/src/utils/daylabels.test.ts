import { describe, it, expect, beforeEach } from 'vitest';
import { getDayLabel, getDayLabels, setDayLabel, daysUntil, formatCountdown } from './daylabels.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach((k) => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('day labels', () => {
  beforeEach(() => localStorage.clear());

  it('stores, trims, and deletes labels', () => {
    expect(setDayLabel('2026-10-05', '  Offsite  ')).toBe('Offsite');
    expect(getDayLabel('2026-10-05')).toBe('Offsite');
    expect(setDayLabel('2026-10-05', '   ')).toBeNull();
    expect(getDayLabel('2026-10-05')).toBeNull();
  });

  it('ignores malformed persisted data', () => {
    mockStorage['kestrel:calendar:day-labels'] = '["not-an-object"]';
    expect(getDayLabel('2026-10-05')).toBeNull();
  });

  it('counts days from local today', () => {
    const now = new Date(2026, 9, 1, 12, 0, 0);
    expect(daysUntil('2026-10-01', now)).toBe(0);
    expect(daysUntil('2026-10-06', now)).toBe(5);
    expect(daysUntil('2026-09-28', now)).toBe(-3);
  });

  it('formats countdowns', () => {
    const now = new Date(2026, 9, 1, 12, 0, 0);
    expect(formatCountdown('2026-10-01', now)).toBe('today');
    expect(formatCountdown('2026-10-02', now)).toBe('tomorrow');
    expect(formatCountdown('2026-10-09', now)).toBe('in 8 days');
    expect(formatCountdown('2026-09-28', now)).toBeNull();
  });

  it('never shows today for garbage input', () => {
    expect(Number.isNaN(daysUntil('', new Date()))).toBe(true);
    expect(Number.isNaN(daysUntil('not-a-date', new Date()))).toBe(true);
    expect(formatCountdown('', new Date())).toBeNull();
    expect(formatCountdown('garbage', new Date())).toBeNull();
  });

  it('truncates long labels and rejects bad keys', () => {
    expect(setDayLabel('2026-10-05', 'x'.repeat(100))).toHaveLength(60);
    expect(setDayLabel('tomorrow', 'Hi')).toBeNull();
    expect(getDayLabel('tomorrow')).toBeNull();
  });

  it('filters malformed persisted entries', () => {
    mockStorage['kestrel:calendar:day-labels'] = JSON.stringify({
      '2026-10-05': 'OK',
      nope: 'bad key',
      '2026-10-06': 42,
    });
    expect(getDayLabels()).toEqual({ '2026-10-05': 'OK' });
  });
});
