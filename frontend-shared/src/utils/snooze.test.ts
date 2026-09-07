import { describe, it, expect } from 'vitest';
import { resolveSnoozeTimestamp, snoozePresetLabel } from './snooze.js';

const DAY = 24 * 3600;

describe('snooze presets', () => {
  it('resolves 1h to one hour ahead', () => {
    const from = new Date(2026, 8, 7, 10, 0).getTime();
    expect(resolveSnoozeTimestamp('1h', from)).toBe(Math.floor(from / 1000) + 3600);
  });

  it('resolves later_today to 6pm or +3h when past', () => {
    const morning = new Date(2026, 8, 7, 10, 0).getTime();
    const sixPm = new Date(2026, 8, 7, 18, 0).getTime();
    expect(resolveSnoozeTimestamp('later_today', morning)).toBe(Math.floor(sixPm / 1000));

    const evening = new Date(2026, 8, 7, 20, 0).getTime();
    expect(resolveSnoozeTimestamp('later_today', evening)).toBe(
      Math.floor(evening / 1000) + 3 * 3600
    );
  });

  it('resolves tomorrow and next week to 9am', () => {
    const monday10am = new Date(2026, 8, 7, 10, 0).getTime();
    const tue9am = new Date(2026, 8, 8, 9, 0).getTime();
    expect(resolveSnoozeTimestamp('tomorrow', monday10am)).toBe(Math.floor(tue9am / 1000));

    const nextMon9am = new Date(2026, 8, 14, 9, 0).getTime();
    expect(resolveSnoozeTimestamp('next_week', monday10am)).toBe(Math.floor(nextMon9am / 1000));
  });

  it('labels presets for UI', () => {
    expect(snoozePresetLabel('tomorrow')).toBe('Tomorrow morning');
  });

  it('keeps presets ordered by delay', () => {
    const from = new Date(2026, 8, 7, 10, 0).getTime();
    const stamps = [
      resolveSnoozeTimestamp('1h', from),
      resolveSnoozeTimestamp('later_today', from),
      resolveSnoozeTimestamp('tomorrow', from),
      resolveSnoozeTimestamp('next_week', from),
    ];
    const diffs = stamps.map((s) => s - Math.floor(from / 1000));
    expect(diffs[0]).toBeLessThan(DAY);
    expect(diffs[3]).toBeGreaterThan(DAY);
  });
});
