import { describe, it, expect } from 'vitest';
import { isNewsletter } from './newsletters.js';

describe('Newsletter detection', () => {
  it('flags provider mail and list markers', () => {
    expect(isNewsletter({ senderEmail: 'n@substack.com', subject: 'Weekly' })).toBe(true);
    expect(isNewsletter({ senderEmail: 's@shop.com', subject: 'News', snippet: 'Unsubscribe here' })).toBe(true);
    expect(isNewsletter({ senderEmail: 'boss@co.com', subject: 'Q3', snippet: 'See attached' })).toBe(false);
  });

  it('rejects lookalike domains', () => {
    expect(isNewsletter({ senderEmail: 'n@xsubstack.com', subject: 'Hi' })).toBe(false);
  });
});
