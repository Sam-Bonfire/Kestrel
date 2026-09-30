import { describe, it, expect, beforeEach } from 'vitest';
import { firstTimeSenders, approveSender, loadScreened, screenedKey } from './screener.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

const msgs = [
  { id: 'm1', senderEmail: 'new@example.com', sender: 'New Guy', subject: 'Hello', timestamp: '2026-09-01T10:00:00Z' },
  { id: 'm2', senderEmail: 'regular@example.com', sender: 'Regular', subject: 'One', timestamp: '2026-09-01T09:00:00Z' },
  { id: 'm3', senderEmail: 'regular@example.com', sender: 'Regular', subject: 'Two', timestamp: '2026-09-01T08:00:00Z' },
];

describe('Screener queue', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('lists senders with exactly one message', () => {
    const queue = firstTimeSenders(msgs, new Set());
    expect(queue.map((q) => q.email)).toEqual(['new@example.com']);
    expect(queue[0].messageId).toBe('m1');
  });

  it('hides approved senders', () => {
    const screened = approveSender('new@example.com', new Set());
    expect(firstTimeSenders(msgs, screened)).toEqual([]);
    expect(loadScreened().has('new@example.com')).toBe(true);
  });

  it('matches case-insensitively', () => {
    const screened = approveSender('NEW@EXAMPLE.COM', new Set());
    expect(firstTimeSenders(msgs, screened)).toEqual([]);
  });

  it('orders newest first', () => {
    const more = [
      ...msgs,
      { id: 'm4', senderEmail: 'fresh@example.com', sender: 'Fresh', subject: 'Yo', timestamp: '2026-09-02T10:00:00Z' },
    ];
    expect(firstTimeSenders(more, new Set()).map((q) => q.email)).toEqual([
      'fresh@example.com',
      'new@example.com',
    ]);
  });

  it('ignores messages without a sender email', () => {
    const withBlank = [
      ...msgs,
      { id: 'm5', senderEmail: '', sender: 'No Address', subject: '?', timestamp: '2026-09-03T10:00:00Z' },
      { id: 'm6', senderEmail: '   ', sender: 'Spaces', subject: '?', timestamp: '2026-09-03T09:00:00Z' },
    ];
    expect(firstTimeSenders(withBlank, new Set()).map((q) => q.email)).toEqual(['new@example.com']);
  });

  it('scopes review keys per account', () => {
    expect(screenedKey('A@x.com', 'acct1')).toBe('acct1:a@x.com');
    expect(screenedKey('A@x.com', '')).toBe('a@x.com');
    expect(screenedKey('  ', 'acct1')).toBe('');
    const scoped = approveSender('new@example.com', new Set(), 'acct1');
    expect(firstTimeSenders(msgs, scoped, 'acct1')).toEqual([]);
    expect(firstTimeSenders(msgs, scoped, 'acct2')).toHaveLength(1);
  });
});
