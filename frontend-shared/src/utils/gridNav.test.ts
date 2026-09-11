import { describe, it, expect, beforeEach } from 'vitest';
import { focusDayColumn, focusEventInColumn } from './gridNav.js';

function setupGrid() {
  document.body.innerHTML = `
    <div data-day-grid>
      <div data-day-col="2026-09-07" tabindex="0">
        <button data-event-card>Standup</button>
        <button data-event-card>Lunch</button>
      </div>
      <div data-day-col="2026-09-08" tabindex="0">
        <button data-event-card>Review</button>
      </div>
      <div data-day-col="2026-09-09" tabindex="0"></div>
    </div>`;
  return document.querySelector('[data-day-grid]') as HTMLElement;
}

describe('Grid keyboard navigation', () => {
  beforeEach(() => {
    setupGrid();
  });

  it('moves focus between day columns with wraparound', () => {
    const grid = document.querySelector('[data-day-grid]') as HTMLElement;
    const cols = [...document.querySelectorAll<HTMLElement>('[data-day-col]')];
    cols[0].focus();
    focusDayColumn(grid, 1);
    expect(document.activeElement).toBe(cols[1]);
    focusDayColumn(grid, 1);
    expect(document.activeElement).toBe(cols[2]);
    focusDayColumn(grid, 1);
    expect(document.activeElement).toBe(cols[0]);
    focusDayColumn(grid, -1);
    expect(document.activeElement).toBe(cols[2]);
  });

  it('moves between event cards within a column', () => {
    const cols = [...document.querySelectorAll<HTMLElement>('[data-day-col]')];
    const cards = [...cols[0].querySelectorAll<HTMLElement>('[data-event-card]')];
    cards[0].focus();
    focusEventInColumn(cols[0], 1);
    expect(document.activeElement).toBe(cards[1]);
    focusEventInColumn(cols[0], 1);
    expect(document.activeElement).toBe(cards[0]);
    focusEventInColumn(cols[0], -1);
    expect(document.activeElement).toBe(cards[1]);
  });

  it('jumps to first and last cards', () => {
    const cols = [...document.querySelectorAll<HTMLElement>('[data-day-col]')];
    const cards = [...cols[0].querySelectorAll<HTMLElement>('[data-event-card]')];
    focusEventInColumn(cols[0], 'last');
    expect(document.activeElement).toBe(cards[1]);
    focusEventInColumn(cols[0], 'first');
    expect(document.activeElement).toBe(cards[0]);
  });

  it('returns false with nothing to focus', () => {
    const grid = document.querySelector('[data-day-grid]') as HTMLElement;
    const cols = [...document.querySelectorAll<HTMLElement>('[data-day-col]')];
    expect(focusEventInColumn(cols[2], 1)).toBe(false);
    const empty = document.createElement('div');
    expect(focusDayColumn(empty, 1)).toBe(false);
  });
});
