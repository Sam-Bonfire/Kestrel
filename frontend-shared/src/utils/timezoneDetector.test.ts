import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { detectTimezone } from './timezoneDetector.js';

describe('timezoneDetector', () => {
  const originalTz = process.env.TZ;

  beforeEach(() => {
    process.env.TZ = 'UTC';
    vi.spyOn(Intl.DateTimeFormat.prototype, 'resolvedOptions').mockReturnValue({
      timeZone: 'UTC',
      locale: 'en-US',
      calendar: 'gregory',
      numberingSystem: 'latn'
    } as unknown as Intl.ResolvedDateTimeFormatOptions);
  });

  afterEach(() => {
    process.env.TZ = originalTz;
    vi.restoreAllMocks();
  });

  it('detects 12-hour time with timezone (EST)', () => {
    const result = detectTimezone('Let\'s meet at 3:00 PM EST tomorrow.');
    expect(result).not.toBeNull();
    if (result) {
      expect(result.originalText).toBe('3:00 PM EST');
      expect(result.sourceTime).toBe('3:00 PM');
      expect(result.sourceTimezone).toBe('EST');
      expect(result.formattedLocalTime).toMatch(/^(7:00 PM|8:00 PM)$/);
    }
  });

  it('detects 12-hour time without minutes (PDT)', () => {
    const result = detectTimezone('Call me at 10am PDT');
    expect(result).not.toBeNull();
    if (result) {
      expect(result.originalText).toMatch(/10am PDT/i);
      expect(result.sourceTime).toBe('10am');
      expect(result.sourceTimezone).toBe('PDT');
      expect(result.formattedLocalTime).toMatch(/^(5:00 PM|6:00 PM)$/);
    }
  });

  it('detects 24-hour time with timezone (JST)', () => {
    const result = detectTimezone('Server restart at 14:00 JST');
    expect(result).not.toBeNull();
    if (result) {
      expect(result.originalText).toMatch(/14:00 JST/i);
      expect(result.sourceTime).toBe('14:00');
      expect(result.sourceTimezone).toBe('JST');
      expect(result.formattedLocalTime).toBe('5:00 AM');
    }
  });

  it('returns null for same timezone as user', () => {
    const result = detectTimezone('Meeting at 14:00 UTC');
    expect(result).toBeNull();
  });

  it('returns null for no match', () => {
    expect(detectTimezone('Meeting at 3pm')).toBeNull();
    expect(detectTimezone('No time here')).toBeNull();
  });
});
