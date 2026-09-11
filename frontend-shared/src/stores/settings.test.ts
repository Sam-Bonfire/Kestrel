import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { mailDensity, mailDenseMode, resolveDensity } from './settings.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Mail density modes', () => {
  beforeEach(() => {
    localStorage.clear();
    mailDensity.set('comfortable');
  });

  it('defaults to comfortable', () => {
    expect(['compact', 'comfortable', 'roomy']).toContain(get(mailDensity));
  });

  it('mirrors compact onto the legacy dense flag for backend sync', () => {
    mailDensity.set('compact');
    expect(get(mailDenseMode)).toBe(true);
    mailDensity.set('roomy');
    expect(get(mailDenseMode)).toBe(false);
  });

  it('persists the 3-way choice', () => {
    mailDensity.set('roomy');
    expect(localStorage.getItem('kestrel:settings:density')).toBe('roomy');
  });

  it('migrates the legacy boolean toggle once', () => {
    expect(resolveDensity(null, 'true')).toBe('compact');
    expect(resolveDensity(null, 'false')).toBe('comfortable');
    expect(resolveDensity('roomy', 'true')).toBe('roomy');
    expect(resolveDensity(null, null)).toBe('comfortable');
    expect(resolveDensity('bogus', null)).toBe('comfortable');
  });
});
