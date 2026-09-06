import { describe, it, expect } from 'vitest';
import {
  DEFAULT_WORKING_HOURS,
  isWorkingDay,
  isWorkingHour,
  parseTimeToMinutes,
  type WorkingHoursConfig,
} from './workingHours.js';

const monday10am = new Date(2026, 8, 7, 10, 0); // Mon Sep 7 2026
const monday8am = new Date(2026, 8, 7, 8, 0);
const saturday12pm = new Date(2026, 8, 12, 12, 0);

describe('workingHours', () => {
  it('parses HH:mm to minutes since midnight', () => {
    expect(parseTimeToMinutes('09:00')).toBe(540);
    expect(parseTimeToMinutes('17:30')).toBe(1050);
  });

  it('recognizes configured working days', () => {
    expect(isWorkingDay(monday10am, DEFAULT_WORKING_HOURS)).toBe(true);
    expect(isWorkingDay(saturday12pm, DEFAULT_WORKING_HOURS)).toBe(false);
  });

  it('recognizes working hours within the window', () => {
    expect(isWorkingHour(monday10am, DEFAULT_WORKING_HOURS)).toBe(true);
    expect(isWorkingHour(monday8am, DEFAULT_WORKING_HOURS)).toBe(false);
    expect(isWorkingHour(saturday12pm, DEFAULT_WORKING_HOURS)).toBe(false);
  });

  it('treats everything as non-working when disabled', () => {
    const disabled: WorkingHoursConfig = { ...DEFAULT_WORKING_HOURS, enabled: false };
    expect(isWorkingHour(monday10am, disabled)).toBe(false);
  });
});
