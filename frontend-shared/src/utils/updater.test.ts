import { describe, it, expect } from 'vitest';
import { describeUpdateError } from './updater.js';

describe('describeUpdateError', () => {
  it('translates JSON manifest failures into actionable copy', () => {
    const out = describeUpdateError(new Error('JSON error: expected value at line 1 column 1'));
    expect(out).not.toMatch(/expected value/i);
    expect(out).toMatch(/no update manifest/i);
  });

  it('translates missing-manifest responses', () => {
    expect(describeUpdateError(new Error('HTTP 404'))).toMatch(/no update manifest/i);
  });

  it('passes through unrelated errors', () => {
    expect(describeUpdateError(new Error('signature mismatch'))).toBe('signature mismatch');
  });

  it('handles non-Error values', () => {
    expect(describeUpdateError('boom')).toBe('boom');
    expect(describeUpdateError(undefined)).toMatch(/try again/i);
  });
});
