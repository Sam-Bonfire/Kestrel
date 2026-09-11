import { describe, it, expect } from 'vitest';
import { applyEventTemplate, builtInEventTemplates } from './eventTemplates.js';

describe('Event templates', () => {
  it('ships usable built-in presets', () => {
    expect(builtInEventTemplates.length).toBeGreaterThanOrEqual(3);
    for (const t of builtInEventTemplates) {
      expect(t.durationMins).toBeGreaterThan(0);
    }
  });

  it('anchors the end time at start plus duration', () => {
    const applied = applyEventTemplate(builtInEventTemplates[0], '09:00');
    expect(applied.endTime).toBe('09:15');
    expect(applied.title).toBe('Standup');
    expect(applied.category).toBe('Work');
  });

  it('wraps past midnight and flags the day overflow', () => {
    const applied = applyEventTemplate(builtInEventTemplates[2], '23:30');
    expect(applied.endTime).toBe('00:30');
    expect(applied.spansNextDay).toBe(true);
    expect(applyEventTemplate(builtInEventTemplates[0], '09:00').spansNextDay).toBe(false);
  });

  it('defaults missing descriptions to empty', () => {
    const applied = applyEventTemplate(builtInEventTemplates[1], '10:00');
    expect(applied.description).toBe('');
  });
});
