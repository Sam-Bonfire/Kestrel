import { describe, it, expect } from 'vitest';
import { inboxCategory } from './inboxCategory.js';

describe('Inbox category tabs', () => {
  it('classifies each tab', () => {
    expect(inboxCategory({ senderEmail: 'a@b.com', subject: 'Hi' })).toBe('Primary');
    expect(inboxCategory({ senderEmail: 'n@github.com', subject: 'PR' })).toBe('Updates');
    expect(inboxCategory({ senderEmail: 'n@linkedin.com', subject: 'Hi' })).toBe('Social');
    expect(inboxCategory({ senderEmail: 's@shop.com', subject: 'Sale', body: 'Unsubscribe' })).toBe('Promotions');
    expect(inboxCategory({ senderEmail: 'l@community.org', subject: 'Digest', body: 'google groups link' })).toBe('Forums');
  });

  it('rejects lookalike domains', () => {
    expect(inboxCategory({ senderEmail: 'a@xgithub.com', subject: 'Hi' })).toBe('Primary');
  });
});
