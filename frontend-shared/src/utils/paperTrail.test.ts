import { describe, it, expect } from 'vitest';
import { isTransactional, resolveBucket } from './paperTrail.js';

describe('isTransactional', () => {
  it('matches receipt subjects', () => {
    expect(isTransactional({ subject: 'Your receipt from Acme' })).toBe(true);
    expect(isTransactional({ subject: 'Invoice #1234 due' })).toBe(true);
    expect(isTransactional({ subject: 'Your booking confirmation' })).toBe(true);
  });

  it('matches receipt senders with order context', () => {
    expect(
      isTransactional({ senderEmail: 'orders@shop.com', subject: 'Thanks for your order!' })
    ).toBe(true);
    expect(isTransactional({ senderEmail: 'orders@shop.com', subject: 'Hello friend' })).toBe(false);
  });

  it('rejects personal mail', () => {
    expect(isTransactional({ senderEmail: 'ana@example.com', subject: 'Lunch Thursday?' })).toBe(false);
    expect(isTransactional({})).toBe(false);
  });

  it('avoids substring traps', () => {
    expect(
      isTransactional({ senderEmail: 'news@noreply.com', subject: 'In order to serve you better' })
    ).toBe(false);
    expect(
      isTransactional({ senderEmail: 'dean@uni.edu', subject: 'Statement of purpose guidelines' })
    ).toBe(false);
    expect(
      isTransactional({ senderEmail: 'borders@bookstore.com', subject: 'Summer sale starts now' })
    ).toBe(false);
    expect(
      isTransactional({ senderEmail: 'supporters@club.org', subject: '2 new updates this week' })
    ).toBe(false);
    expect(isTransactional({ senderEmail: 'ORDERS@SHOP.COM', subject: 'order #42 shipped' })).toBe(
      true
    );
  });

  it('matches in snippet and body text', () => {
    expect(isTransactional({ subject: 'Hi', snippet: 'Find your invoice attached' })).toBe(true);
  });
});

describe('resolveBucket', () => {
  const mail = { senderEmail: 'a@shop.com', subject: 'Your order shipped', labels: ['News'] };

  it('prefers user label pins over everything', () => {
    expect(resolveBucket(mail, { News: 'inbox' }, {})).toBe('inbox');
    expect(resolveBucket(mail, { News: 'feed' }, { 'a@shop.com': 'inbox' })).toBe('feed');
  });

  it('prefers sender pins over the classifier', () => {
    expect(resolveBucket(mail, {}, { 'a@shop.com': 'inbox' })).toBe('inbox');
  });

  it('falls back to the classifier, then null', () => {
    expect(resolveBucket(mail, {}, {})).toBe('paper');
    expect(
      resolveBucket({ senderEmail: 'ana@example.com', subject: 'Hi', labels: [] }, {}, {})
    ).toBeNull();
  });
});
