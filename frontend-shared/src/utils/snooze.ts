export type SnoozePreset = '1h' | 'later_today' | 'tomorrow' | 'next_week';

export const SNOOZE_PRESETS: { value: SnoozePreset; label: string }[] = [
  { value: '1h', label: '1 hour' },
  { value: 'later_today', label: 'Later today' },
  { value: 'tomorrow', label: 'Tomorrow morning' },
  { value: 'next_week', label: 'Next week' },
];

export const DEFAULT_SNOOZE_PRESET: SnoozePreset = '1h';

export function snoozePresetLabel(preset: SnoozePreset): string {
  return SNOOZE_PRESETS.find((p) => p.value === preset)?.label ?? preset;
}

/** Resolve a preset to epoch seconds. Pure function of `fromMs` for testability. */
export function resolveSnoozeTimestamp(preset: SnoozePreset, fromMs: number = Date.now()): number {
  const from = new Date(fromMs);
  switch (preset) {
    case '1h':
      return Math.floor(fromMs / 1000) + 3600;
    case 'later_today': {
      const sixPm = new Date(from);
      sixPm.setHours(18, 0, 0, 0);
      if (sixPm.getTime() > fromMs) return Math.floor(sixPm.getTime() / 1000);
      return Math.floor(fromMs / 1000) + 3 * 3600;
    }
    case 'tomorrow': {
      const morning = new Date(from);
      morning.setDate(morning.getDate() + 1);
      morning.setHours(9, 0, 0, 0);
      return Math.floor(morning.getTime() / 1000);
    }
    case 'next_week': {
      const monday = new Date(from);
      monday.setDate(monday.getDate() + ((8 - monday.getDay()) % 7 || 7));
      monday.setHours(9, 0, 0, 0);
      return Math.floor(monday.getTime() / 1000);
    }
  }
}
