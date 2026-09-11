import { describe, it, expect } from 'vitest';
import { categorizeEmail } from './categorize.js';

describe('Smart categorization rules', () => {
  it('defaults personal mail to Primary', () => {
    expect(categorizeEmail({ senderEmail: 'boss@company.com', subject: 'Q3 planning', snippet: 'Hi, see attached' })).toBe('Primary');
    expect(categorizeEmail({})).toBe('Primary');
  });

  it('keeps sent and draft mail in Primary', () => {
    expect(categorizeEmail({ senderEmail: 'me@x.com', subject: 'Summer sale', labels: ['Sent'] })).toBe('Primary');
    expect(categorizeEmail({ senderEmail: 'me@x.com', subject: 'Hi', labels: ['draft'] })).toBe('Primary');
  });

  it('routes social domains to Social', () => {
    expect(categorizeEmail({ senderEmail: 'notify@linkedin.com', subject: 'You appeared in searches' })).toBe('Social');
    expect(categorizeEmail({ senderEmail: 'noreply@sub.youtube.com', subject: 'New video' })).toBe('Social');
  });

  it('routes dev/ci notifications to Updates', () => {
    expect(categorizeEmail({ senderEmail: 'notifications@github.com', subject: '[repo] PR merged' })).toBe('Updates');
    expect(categorizeEmail({ senderEmail: 'builds@circleci.com', subject: 'Build passed' })).toBe('Updates');
    expect(categorizeEmail({ senderEmail: 'alerts@company.com', subject: 'Deploy', labels: ['ci-passed'] })).toBe('Updates');
  });

  it('rejects lookalike spoof domains', () => {
    expect(categorizeEmail({ senderEmail: 'a@xgithub.com', subject: 'Hi' })).toBe('Primary');
    expect(categorizeEmail({ senderEmail: 'a@github.com.evil.com', subject: 'Hi' })).toBe('Primary');
  });

  it('routes receipts and invites to Updates, not Promotions', () => {
    expect(categorizeEmail({ senderEmail: 'receipts@airline.com', subject: 'Your receipt', snippet: 'Invoice attached' })).toBe('Updates');
    expect(categorizeEmail({ senderEmail: 'cal@company.com', subject: 'Invitation: Planning @ Mon', snippet: 'RSVP requested' })).toBe('Updates');
  });

  it('routes lists with unsubscribe footers to Forums', () => {
    expect(
      categorizeEmail({ senderEmail: 'list@community.org', subject: 'Weekly digest', snippet: 'Unsubscribe: google groups link' })
    ).toBe('Forums');
  });

  it('routes promos to Promotions', () => {
    expect(categorizeEmail({ senderEmail: 'store@shop.com', subject: 'Summer sale: 30% off' })).toBe('Promotions');
  });

  it('prefers Social over keyword matches', () => {
    expect(categorizeEmail({ senderEmail: 'news@facebook.com', subject: 'Summer sale' })).toBe('Social');
  });
});
