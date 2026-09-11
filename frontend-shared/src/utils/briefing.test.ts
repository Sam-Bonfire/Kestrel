import { describe, it, expect, beforeEach } from 'vitest';
import { buildDailyBriefing, shouldShowBriefing, markBriefingShown } from './briefing.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Daily briefing', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('shows once per day', () => {
    const morning = new Date(2026, 8, 9, 8, 0, 0);
    expect(shouldShowBriefing(morning)).toBe(true);
    markBriefingShown(morning);
    expect(shouldShowBriefing(morning)).toBe(false);
    expect(shouldShowBriefing(new Date(2026, 8, 10, 8, 0, 0))).toBe(true);
  });

  it('summarizes agenda and inbox', () => {
    const briefing = buildDailyBriefing({
      events: [
        { title: 'Lunch', startTime: '12:30' },
        { title: 'Standup', startTime: '09:00' },
      ],
      unreadCount: 3,
    });
    expect(briefing.title).toContain('2 events');
    expect(briefing.title).toContain('09:00');
    expect(briefing.body).toContain('09:00 Standup');
    expect(briefing.body).toContain('3 unread messages');
  });

  it('caps the event list and handles calm days', () => {
    const many = Array.from({ length: 7 }, (_, i) => ({ title: `E${i}`, startTime: `0${i}:00` }));
    const capped = buildDailyBriefing({ events: many, unreadCount: 0 });
    expect(capped.body).toContain('+2 more');
    const calm = buildDailyBriefing({ events: [], unreadCount: 0 });
    expect(calm.body).toContain('calm');
  });
});
