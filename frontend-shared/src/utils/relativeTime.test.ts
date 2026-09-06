import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { formatRelativeTime, formatExactDateTime } from './relativeTime.js';

describe('relativeTime utility', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('formatRelativeTime', () => {
    it('returns "Just now" for times under 60 seconds', () => {
      const now = new Date('2023-10-15T12:00:00Z');
      const time = new Date('2023-10-15T11:59:01Z'); // 59 seconds ago
      expect(formatRelativeTime(time, now)).toBe('Just now');
    });

    it('returns minutes for times 60 seconds and over but under 60 minutes', () => {
      const now = new Date('2023-10-15T12:00:00Z');
      const time60s = new Date('2023-10-15T11:59:00Z'); // 60 seconds ago -> 1m
      const time59m = new Date('2023-10-15T11:01:00Z'); // 59 mins ago -> 59m

      expect(formatRelativeTime(time60s, now)).toBe('1m');
      expect(formatRelativeTime(time59m, now)).toBe('59m');
    });

    it('returns time style short for times under 24 hours (same calendar day)', () => {
      const now = new Date('2023-10-15T23:00:00Z');
      const time = new Date('2023-10-15T10:30:00Z'); // Same day

      const expected = new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(time);
      expect(formatRelativeTime(time, now)).toBe(expected);
    });

    it('returns "Yesterday" for previous calendar day', () => {
      const now = new Date('2023-10-15T00:01:00Z');
      const time = new Date('2023-10-14T23:59:00Z'); // Yesterday

      expect(formatRelativeTime(time, now)).toBe('Yesterday');
    });

    it('returns full weekday name for within the last 6 days', () => {
      const now = new Date('2023-10-15T12:00:00Z'); // Let's say it's Sunday
      const time = new Date('2023-10-12T12:00:00Z'); // Thursday (3 days ago)

      const expected = new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(time);
      expect(formatRelativeTime(time, now)).toBe(expected);
    });

    it('returns short month and day for older than 6 days (same calendar year)', () => {
      const now = new Date('2023-10-15T12:00:00Z');
      const time = new Date('2023-08-24T12:00:00Z');

      const expected = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' }).format(time);
      expect(formatRelativeTime(time, now)).toBe(expected);
    });

    it('returns short date format for older calendar years', () => {
      const now = new Date('2024-01-01T12:00:00Z');
      const time = new Date('2023-12-31T23:59:00Z'); // Cross-year boundary

      const expected = new Intl.DateTimeFormat(undefined, { dateStyle: 'short' }).format(time);
      expect(formatRelativeTime(time, now)).toBe(expected);
    });
  });

  describe('formatExactDateTime', () => {
    it('returns exact localized date and time', () => {
      const time = new Date('2023-10-15T12:34:56Z');
      const expected = new Intl.DateTimeFormat(undefined, { dateStyle: 'full', timeStyle: 'long' }).format(time);
      expect(formatExactDateTime(time)).toBe(expected);
    });

    it('returns empty string for invalid or missing date', () => {
      expect(formatExactDateTime('')).toBe('');
    });
  });
});
