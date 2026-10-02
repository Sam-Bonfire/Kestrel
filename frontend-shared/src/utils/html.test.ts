import { describe, it, expect } from 'vitest';
import { plainText, sanitizeEmailBody, sanitizeProviderIcon } from './html.js';

describe('plainText', () => {
  it('strips formatting tags', () => {
    expect(plainText('<b>Bold</b> and <i>italic</i>')).toBe('Bold and italic');
  });

  it('flattens lists and links', () => {
    expect(plainText('<ul><li>one</li><li>two</li></ul>')).toBe('one two');
    expect(plainText('<a href="https://x">click</a>')).toBe('click');
  });

  it('handles empty and plain input', () => {
    expect(plainText('')).toBe('');
    expect(plainText(null)).toBe('');
    expect(plainText(undefined)).toBe('');
    expect(plainText('already plain')).toBe('already plain');
  });
});

describe('sanitizeEmailBody', () => {
  it('removes script tags but keeps formatting', () => {
    const out = sanitizeEmailBody('<p>Hello</p><script>alert(1)</script>');
    expect(out).toContain('Hello');
    expect(out).not.toContain('<script');
  });

  it('strips event handlers and javascript hrefs', () => {
    const out = sanitizeEmailBody('<img src=x onerror=alert(1)><a href="javascript:alert(1)">x</a>');
    expect(out).not.toContain('onerror');
    expect(out).not.toContain('javascript:');
  });

  it('passes plaintext through and handles empty', () => {
    expect(sanitizeEmailBody('plain snippet')).toBe('plain snippet');
    expect(sanitizeEmailBody('')).toBe('');
    expect(sanitizeEmailBody(null)).toBe('');
  });
});

describe('sanitizeProviderIcon', () => {
  it('keeps svg shapes but strips onload handlers', () => {
    const out = sanitizeProviderIcon('<svg onload=alert(1)><circle r="5"/></svg>');
    expect(out).toContain('<circle');
    expect(out).not.toContain('onload');
  });

  it('handles empty', () => {
    expect(sanitizeProviderIcon('')).toBe('');
    expect(sanitizeProviderIcon(null)).toBe('');
  });
});
