import { describe, it, expect } from 'vitest';
import { parseNaturalEvent, shiftTime } from './nlpEvent.js';

// Monday 2026-09-07 10:00 local
const NOW = new Date(2026, 8, 7, 10, 0, 0);

describe('Natural-language event parsing', () => {
  it('rejects empty input', () => {
    expect(parseNaturalEvent('   ', NOW)).toBeNull();
  });

  it('parses relative day, time and duration', () => {
    expect(parseNaturalEvent('lunch tomorrow 12:30 1h', NOW)).toEqual({
      title: 'Lunch',
      date: '2026-09-08',
      startTime: '12:30',
      durationMins: 60,
    });
  });

  it('defaults to 9am for one hour', () => {
    expect(parseNaturalEvent('dentist', NOW)).toEqual({
      title: 'Dentist',
      date: '2026-09-07',
      startTime: '09:00',
      durationMins: 60,
    });
  });

  it('resolves weekday names to the next occurrence', () => {
    expect(parseNaturalEvent('standup friday 9am 15m', NOW)?.date).toBe('2026-09-11');
    expect(parseNaturalEvent('review monday 10:00', NOW)?.date).toBe('2026-09-14');
  });

  it('handles tonight, ranges and explicit dates', () => {
    expect(parseNaturalEvent('movie tonight 8pm', NOW)).toMatchObject({ date: '2026-09-07', startTime: '20:00' });
    expect(parseNaturalEvent('sync 2-3pm', NOW)).toMatchObject({ startTime: '14:00' });
    expect(parseNaturalEvent('conf 2026-10-01 09:00', NOW)?.date).toBe('2026-10-01');
  });

  it('parses 24h times and minute durations', () => {
    expect(parseNaturalEvent('call 15:00 30m', NOW)).toMatchObject({ startTime: '15:00', durationMins: 30 });
  });

  it('sums compound durations and clamps the range', () => {
    expect(parseNaturalEvent('workshop 1h30m', NOW)).toMatchObject({ durationMins: 90, startTime: '09:00' });
    expect(parseNaturalEvent('x 0m', NOW)).toMatchObject({ durationMins: 1 });
  });

  it('leaves invalid times in the title', () => {
    expect(parseNaturalEvent('party 25:00', NOW)).toMatchObject({ title: 'Party 25:00', startTime: '09:00' });
  });

  it('keeps "in N hours" phrases as titles', () => {
    expect(parseNaturalEvent('meeting in 2 hours', NOW)).toMatchObject({ title: 'Meeting in 2 hours' });
  });

  it('shifts times across day boundaries', () => {
    expect(shiftTime('23:30', 60)).toEqual({ endTime: '00:30', daysAfter: 1 });
    expect(shiftTime('09:00', 60)).toEqual({ endTime: '10:00', daysAfter: 0 });
  });
});
