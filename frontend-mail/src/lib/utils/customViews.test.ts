import { describe, it, expect, beforeEach } from 'vitest';
import { getCustomViews, saveCustomView, deleteCustomView, updateCustomView } from './customViews.js';
import type { CustomViewFilter } from './customViews.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

const filter: CustomViewFilter = {
  view: 'inbox',
  category: 'Updates',
  label: 'All',
  attachmentOnly: false,
  dateRange: 'This Week',
  unreadOnly: true,
};

describe('Custom inbox views', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('saves and lists views', () => {
    expect(getCustomViews()).toEqual([]);
    const next = saveCustomView('Morning triage', filter);
    expect(next).toHaveLength(1);
    expect(next[0].name).toBe('Morning triage');
    expect(next[0].filter).toEqual(filter);
    expect(getCustomViews()).toHaveLength(1);
  });

  it('rejects blank names', () => {
    expect(saveCustomView('   ', filter)).toEqual([]);
  });

  it('deletes by id', () => {
    const [view] = saveCustomView('Temp', filter);
    expect(deleteCustomView(view.id)).toEqual([]);
    expect(getCustomViews()).toEqual([]);
  });

  it('drops corrupted stored entries', () => {
    localStorage.setItem('kestrel:mail:custom_views', JSON.stringify([{ junk: 1 }, null, 'x']));
    expect(getCustomViews()).toEqual([]);
  });

  it('stores an icon when provided', () => {
    const [view] = saveCustomView('Starred work', filter, 'Star');
    expect(view.icon).toBe('Star');
    expect(getCustomViews()[0].icon).toBe('Star');
  });

  it('renames a view', () => {
    const [view] = saveCustomView('Temp', filter);
    const next = updateCustomView(view.id, { name: 'Morning triage' });
    expect(next[0].name).toBe('Morning triage');
    expect(getCustomViews()[0].name).toBe('Morning triage');
  });

  it('changes a view icon', () => {
    const [view] = saveCustomView('Temp', filter);
    const next = updateCustomView(view.id, { icon: 'Bell' });
    expect(next[0].icon).toBe('Bell');
  });

  it('rejects blank renames and unknown ids', () => {
    const [view] = saveCustomView('Temp', filter);
    expect(updateCustomView(view.id, { name: '   ' })[0].name).toBe('Temp');
    expect(updateCustomView('nope', { name: 'X' })).toHaveLength(1);
  });
});
