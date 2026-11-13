import { describe, it, expect, beforeEach } from 'vitest';
import {
  stagePopoutDraft,
  takePopoutDraft,
  popoutDraftNonce,
  buildPopoutUrl,
} from './popout.js';

describe('Compose pop-out handoff', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('stages and takes a draft exactly once', () => {
    const nonce = stagePopoutDraft({ to: ['a@b.com'], subject: 'Hi', body: '<p>x</p>' });
    expect(typeof nonce).toBe('string');
    expect(takePopoutDraft(nonce)).toEqual({ to: ['a@b.com'], subject: 'Hi', body: '<p>x</p>' });
    expect(takePopoutDraft(nonce)).toBeNull();
  });

  it('returns null for unknown nonces', () => {
    expect(takePopoutDraft('nope')).toBeNull();
  });

  it('detects pop-out windows by query', () => {
    expect(popoutDraftNonce('https://app/?popout=compose&draft=abc')).toBe('abc');
    expect(popoutDraftNonce('https://app/?popout=compose')).toBeNull();
    expect(popoutDraftNonce('https://app/?other=1')).toBeNull();
    expect(popoutDraftNonce('garbage')).toBeNull();
  });

  it('builds urls the detector accepts', () => {
    const url = buildPopoutUrl('n-1');
    expect(url).toContain('popout=compose');
    expect(popoutDraftNonce(`https://app/${url}`)).toBe('n-1');
  });
});
