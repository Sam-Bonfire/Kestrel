import { describe, it, expect, beforeEach } from 'vitest';
import { triageCandidates, shouldRunTriage, markTriageRun } from './triage.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

const NOW = new Date('2026-09-09T12:00:00Z');
const OLD = '2026-07-01T10:00:00Z';
const RECENT = '2026-09-08T10:00:00Z';

const base = (overrides = {}) => ({
  id: 'm1',
  isUnread: false,
  isStarred: false,
  isArchived: false,
  isTrash: false,
  category: 'Promotions',
  timestamp: OLD,
  ...overrides,
});

describe('Smart triage', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('runs once per day', () => {
    expect(shouldRunTriage(NOW)).toBe(true);
    markTriageRun(NOW);
    expect(shouldRunTriage(NOW)).toBe(false);
    expect(shouldRunTriage(new Date('2026-09-10T08:00:00Z'))).toBe(true);
  });

  it('picks old read promos and social', () => {
    const ids = triageCandidates(
      [base({ id: 'a' }), base({ id: 'b', category: 'Social' }), base({ id: 'c', category: 'Primary' })],
      NOW
    );
    expect(ids).toEqual(['a', 'b']);
  });

  it('skips unread, starred, fresh, trashed and archived mail', () => {
    const ids = triageCandidates(
      [
        base({ id: 'u', isUnread: true }),
        base({ id: 's', isStarred: true }),
        base({ id: 'r', timestamp: RECENT }),
        base({ id: 't', isTrash: true }),
        base({ id: 'a', isArchived: true }),
        base({ id: 'n' }),
      ],
      NOW
    );
    expect(ids).toEqual(['n']);
  });

  it('falls back to bulk-mail markers without a usable category', () => {
    const ids = triageCandidates(
      [
        { id: 'p', isUnread: false, timestamp: OLD, subject: 'Weekly deals', snippet: 'Unsubscribe here' },
        { id: 'q', isUnread: false, timestamp: OLD, subject: 'Big sale numbers', snippet: 'See attached' },
        { id: 'r', isUnread: false, timestamp: OLD, subject: 'Your receipt', snippet: 'Thanks for shopping' },
      ],
      NOW
    );
    expect(ids).toEqual(['p']);
  });
});
