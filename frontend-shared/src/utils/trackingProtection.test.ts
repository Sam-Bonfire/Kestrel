import { describe, it, expect, beforeEach } from 'vitest';
import {
  senderDomain,
  isDomainAllowed,
  allowSenderDomain,
  blockSenderDomain,
  blockRemoteImages,
  hasRemoteContent,
} from './trackingProtection.js';

const ALLOW_KEY = 'kestrel:privacy:remote-image-domains';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Tracking protection', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('extracts sender domains', () => {
    expect(senderDomain('alice@example.com')).toBe('example.com');
    expect(senderDomain('Bob@News.Example.ORG')).toBe('news.example.org');
    expect(senderDomain('not-an-email')).toBe('');
  });

  it('whitelists sender domains', () => {
    expect(isDomainAllowed('example.com')).toBe(false);
    allowSenderDomain('alice@example.com');
    expect(isDomainAllowed('example.com')).toBe(true);
    expect(localStorage.getItem(ALLOW_KEY)).toContain('example.com');
    blockSenderDomain('alice@example.com');
    expect(isDomainAllowed('example.com')).toBe(false);
  });

  it('detects remote content', () => {
    expect(hasRemoteContent('<p>hi <img src="https://tracker.example/pixel.gif"></p>')).toBe(true);
    expect(hasRemoteContent('<p>hi <img src="cid:logo"></p>')).toBe(false);
    expect(hasRemoteContent('<p>plain</p>')).toBe(false);
  });

  it('blocks remote images but keeps cid and data images', () => {
    const { html, blockedCount } = blockRemoteImages(
      '<p>a<img src="https://tracker.example/p.gif" width="1" height="1">b<img src="cid:logo">c<img src="data:image/png;base64,xx"></p>'
    );
    expect(blockedCount).toBe(1);
    expect(html).not.toContain('https://tracker.example/p.gif');
    expect(html).toContain('cid:logo');
    expect(html).toContain('data:image/png;base64,xx');
  });

  it('strips remote background urls', () => {
    const { html, blockedCount } = blockRemoteImages(
      '<div style="background-image: url(https://tracker.example/bg.png)">x</div>'
    );
    expect(blockedCount).toBe(1);
    expect(html).not.toContain('https://tracker.example/bg.png');
  });

  it('closes protocol-relative, style-tag, srcset and background leaks', () => {
    const { html, blockedCount } = blockRemoteImages(
      '<style>.a{background:url(//tracker.example/s.css)}</style>' +
        '<p><img src="//tracker.example/a.gif" srcset="https://tracker.example/b.gif 2x"></p>' +
        '<table background="https://tracker.example/c.gif"><tr><td>x</td></tr></table>'
    );
    expect(html).not.toContain('tracker.example');
    expect(blockedCount).toBeGreaterThanOrEqual(3);
  });
});
