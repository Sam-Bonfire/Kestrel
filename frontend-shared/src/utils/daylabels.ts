const DAY_LABELS_KEY = 'kestrel:calendar:day-labels';
const MAX_LABEL = 60;

function readAll(): Record<string, string> {
  try {
    if (typeof localStorage === 'undefined') return {};
    const raw = localStorage.getItem(DAY_LABELS_KEY);
    const parsed = raw !== null ? (JSON.parse(raw) as unknown) : {};
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) return {};
    const clean: Record<string, string> = {};
    for (const [k, v] of Object.entries(parsed as Record<string, unknown>)) {
      if (typeof v === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(k)) clean[k] = v.slice(0, MAX_LABEL);
    }
    return clean;
  } catch {
    return {};
  }
}

function writeAll(all: Record<string, string>) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(DAY_LABELS_KEY, JSON.stringify(all));
    }
  } catch {
    // Non-fatal.
  }
}

export function getDayLabel(dateStr: string): string | null {
  return readAll()[dateStr] ?? null;
}

export function getDayLabels(): Record<string, string> {
  return readAll();
}

/** Empty text deletes the label. Returns the stored value or null. */
export function setDayLabel(dateStr: string, text: string): string | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) return null;
  const all = readAll();
  const trimmed = text.trim().slice(0, MAX_LABEL);
  if (!trimmed) delete all[dateStr];
  else all[dateStr] = trimmed;
  writeAll(all);
  return all[dateStr] ?? null;
}

/** Whole days from today (local) to the date. Negative = past, NaN = invalid. */
export function daysUntil(dateStr: string, now = new Date()): number {
  const [y, m, d] = dateStr.split('-').map(Number);
  if (!y || !m || !d) return NaN;
  const day = new Date(y, m - 1, d);
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  return Math.round((day.getTime() - today.getTime()) / 86400000);
}

export function formatCountdown(dateStr: string, now = new Date()): string | null {
  const n = daysUntil(dateStr, now);
  if (!Number.isFinite(n) || n < 0) return null;
  if (n === 0) return 'today';
  if (n === 1) return 'tomorrow';
  return `in ${n} days`;
}
