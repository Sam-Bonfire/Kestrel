import { describe, it, expect } from 'vitest';
import { plainText } from './html.js';

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
