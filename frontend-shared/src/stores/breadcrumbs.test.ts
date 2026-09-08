import { describe, it, expect, beforeEach } from 'vitest';
import {
  recentBreadcrumbs,
  pushBreadcrumb,
  clearBreadcrumbs,
  humanizeCrumb,
  MAX_CRUMBS,
} from './breadcrumbs.js';
import { get } from 'svelte/store';

// In-memory localStorage mock for node test environment
const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Recent activity breadcrumbs', () => {
  beforeEach(() => {
    clearBreadcrumbs();
    localStorage.clear();
  });

  it('pushes the newest crumb to the front', () => {
    pushBreadcrumb('Inbox');
    pushBreadcrumb('Starred');
    expect(get(recentBreadcrumbs).map((c) => c.label)).toEqual(['Starred', 'Inbox']);
  });

  it('moves a revisited label to the front instead of duplicating', () => {
    pushBreadcrumb('Inbox');
    pushBreadcrumb('Starred');
    pushBreadcrumb('Inbox');
    expect(get(recentBreadcrumbs).map((c) => c.label)).toEqual(['Inbox', 'Starred']);
  });

  it('caps the trail at MAX_CRUMBS entries', () => {
    for (let i = 0; i < MAX_CRUMBS + 3; i++) pushBreadcrumb(`View ${i}`);
    const crumbs = get(recentBreadcrumbs);
    expect(crumbs).toHaveLength(MAX_CRUMBS);
    expect(crumbs[0].label).toBe(`View ${MAX_CRUMBS + 2}`);
  });

  it('clears the trail', () => {
    pushBreadcrumb('Inbox');
    clearBreadcrumbs();
    expect(get(recentBreadcrumbs)).toEqual([]);
  });

  it('persists the trail across reloads', () => {
    pushBreadcrumb('Inbox');
    const raw = localStorage.getItem('kestrel:nav:recent');
    expect(raw).toContain('Inbox');
  });

  it('humanizes raw view ids for display', () => {
    expect(humanizeCrumb('reply-later')).toBe('Reply later');
    expect(humanizeCrumb('label-receipts')).toBe('Receipts');
    expect(humanizeCrumb('inbox')).toBe('Inbox');
  });
});
