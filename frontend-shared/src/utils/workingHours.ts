export interface WorkingHoursConfig {
  enabled: boolean;
  /** HH:mm 24-hour format, e.g. '09:00' */
  startTime: string;
  /** HH:mm 24-hour format, e.g. '17:00' */
  endTime: string;
  /** 0=Sunday … 6=Saturday */
  daysOfWeek: number[];
}

export const DEFAULT_WORKING_HOURS: WorkingHoursConfig = {
  enabled: true,
  startTime: '09:00',
  endTime: '17:00',
  daysOfWeek: [1, 2, 3, 4, 5],
};

export function parseTimeToMinutes(time: string): number {
  const [h, m] = time.split(':').map(Number);
  return h * 60 + (m || 0);
}

export function isWorkingDay(date: Date, config: WorkingHoursConfig): boolean {
  return config.daysOfWeek.includes(date.getDay());
}

/** True when the config is enabled, the date falls on a working day, and the time is within [start, end). */
export function isWorkingHour(date: Date, config: WorkingHoursConfig): boolean {
  if (!config.enabled || !isWorkingDay(date, config)) return false;
  const mins = date.getHours() * 60 + date.getMinutes();
  return mins >= parseTimeToMinutes(config.startTime) && mins < parseTimeToMinutes(config.endTime);
}
