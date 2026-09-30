import { describe, it, expect } from 'vitest';
import { computeFreeBlocks, formatFreeBlock } from './freetime.js';

describe('computeFreeBlocks', () => {
  it('returns the whole day when empty', () => {
    expect(computeFreeBlocks([])).toEqual([{ startMins: 0, endMins: 1440, mins: 1440 }]);
  });

  it('merges overlapping busy spans', () => {
    const blocks = computeFreeBlocks([
      { startTime: '09:00', endTime: '10:00' },
      { startTime: '09:30', endTime: '11:00' },
    ]);
    expect(blocks[0]).toEqual({ startMins: 660, endMins: 1440, mins: 780 });
  });

  it('drops gaps below the threshold', () => {
    const blocks = computeFreeBlocks(
      [
        { startTime: '09:00', endTime: '10:00' },
        { startTime: '11:00', endTime: '12:00' },
      ],
      120,
    );
    expect(blocks.some((b) => b.startMins === 600)).toBe(false);
    expect(blocks[0].startMins).toBe(720);
  });

  it('sorts longest first', () => {
    const blocks = computeFreeBlocks([{ startTime: '12:00', endTime: '13:00' }]);
    expect(blocks[0].mins).toBeGreaterThanOrEqual(blocks[1].mins);
  });

  it('clips overnight spans instead of dropping them', () => {
    const blocks = computeFreeBlocks([{ startTime: '22:00', endTime: '02:00' }]);
    expect(blocks).toEqual([{ startMins: 0, endMins: 1320, mins: 1320 }]);
  });

  it('yields nothing on a fully busy day', () => {
    expect(computeFreeBlocks([{ startTime: '00:00', endTime: '23:59' }])).toEqual([]);
  });

  it('formats durations', () => {
    expect(formatFreeBlock(180)).toBe('3h free');
    expect(formatFreeBlock(90)).toBe('1h 30m free');
    expect(formatFreeBlock(45)).toBe('45m free');
  });
});
