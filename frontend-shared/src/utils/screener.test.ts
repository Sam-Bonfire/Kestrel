import { describe, it, expect, beforeEach } from 'vitest';
import { firstTimeSenders, approveSender, loadScreened } from './screener.js';

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
});
