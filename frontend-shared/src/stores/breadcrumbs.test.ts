import { describe, it, expect, beforeEach } from 'vitest';
import { currentCrumb, setCurrentCrumb, humanizeCrumb } from './breadcrumbs.js';
import { get } from 'svelte/store';

describe('Location breadcrumb', () => {
  beforeEach(() => {
    setCurrentCrumb('inbox');
  });

  it('defaults to inbox', () => {
    expect(get(currentCrumb)).toBe('inbox');
  });

  it('tracks the current view exactly', () => {
    setCurrentCrumb('week');
    expect(get(currentCrumb)).toBe('week');
    setCurrentCrumb('inbox');
    expect(get(currentCrumb)).toBe('inbox');
  });

  it('humanizes raw view ids for display', () => {
    expect(humanizeCrumb('reply-later')).toBe('Reply later');
    expect(humanizeCrumb('label-receipts')).toBe('Receipts');
    expect(humanizeCrumb('inbox')).toBe('Inbox');
  });
});
