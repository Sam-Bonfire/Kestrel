export function formatRelativeTime(dateInput: string | number | Date, referenceDate?: Date): string {
  const date = new Date(dateInput);
  const now = referenceDate ? new Date(referenceDate) : new Date();

  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);

  const dateDay = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  const nowDay = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  const diffDays = Math.floor((nowDay.getTime() - dateDay.getTime()) / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    if (diffSecs < 60) {
      return 'Just now';
    }

    if (diffMins < 60) {
      return `${diffMins}m`;
    }
  }

  if (diffDays === 0) {
    // Under 24 hours (same calendar day): "14:30" or "2:30 PM" depending on locale
    return new Intl.DateTimeFormat(undefined, { timeStyle: 'short' }).format(date);
  }

  if (date.getFullYear() === now.getFullYear()) {
    if (diffDays === 1) {
      return 'Yesterday';
    }

    if (diffDays > 1 && diffDays <= 6) {
      // Within the last 6 days: Full weekday name
      return new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(date);
    }
    // Older than 6 days (same calendar year): Short month and day
    return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' }).format(date);
  } else {
    // Check if it's the previous day in a different calendar year
    // diffDays alone isn't reliable for "Yesterday" if we want "Older calendar years" to strictly be short date.
    // Actually, "Older calendar years: Short date format" overrides Yesterday if it crosses a year boundary.
    // Wait, the requirements say: "Older calendar years: Short date format".
    // So even if it was yesterday (Dec 31 vs Jan 1), it should return short date format because it's an older calendar year!
    return new Intl.DateTimeFormat(undefined, { dateStyle: 'short' }).format(date);
  }

  // Older calendar years: Short date format (e.g., 08/14/25)
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'short' }).format(date);
}

export function formatExactDateTime(dateInput: string | number | Date): string {
  if (!dateInput) return '';
  const date = new Date(dateInput);
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'full', timeStyle: 'long' }).format(date);
}
