/**
 * Keyboard grid navigation helpers (roving focus across day columns
 * and event cards). Pure DOM queries so both the component and tests
 * share one implementation.
 */
export function focusDayColumn(grid: Element, direction: 1 | -1): boolean {
  const columns = [...grid.querySelectorAll<HTMLElement>('[data-day-col]')];
  if (columns.length === 0) return false;
  const current = document.activeElement as HTMLElement | null;
  const idx = current ? columns.indexOf(current.closest?.('[data-day-col]') as HTMLElement) : -1;
  columns[(idx + direction + columns.length) % columns.length].focus();
  return true;
}

export function focusEventInColumn(
  column: Element,
  direction: 1 | -1 | 'first' | 'last'
): boolean {
  const cards = [...column.querySelectorAll<HTMLElement>('[data-event-card]')];
  if (cards.length === 0) return false;
  if (direction === 'first') {
    cards[0].focus();
    return true;
  }
  if (direction === 'last') {
    cards[cards.length - 1].focus();
    return true;
  }
  const active = document.activeElement as HTMLElement | null;
  const idx = active ? cards.indexOf(active) : -1;
  cards[idx === -1 ? (direction === 1 ? 0 : cards.length - 1) : (idx + direction + cards.length) % cards.length].focus();
  return true;
}
