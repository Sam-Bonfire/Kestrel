import { describe, it, expect } from 'vitest';
import { buildMailFrameSrcdoc } from './mail-frame.js';

function csp(srcdoc: string): string {
  return srcdoc.match(/content="([^"]+)"/)?.[1] ?? '';
}

describe('buildMailFrameSrcdoc', () => {
  it('denies everything by default and only runs the nonced bridge', () => {
    const srcdoc = buildMailFrameSrcdoc('<p>hi</p>', false);
    const header = csp(srcdoc);
    expect(header).toContain("default-src 'none'");
    expect(header).toContain("img-src 'none'");
    expect(header).not.toContain("script-src 'unsafe-inline'");
    expect(header).toMatch(/script-src 'nonce-[^']+'/);
    expect(srcdoc).toContain('<p>hi</p>');
  });

  it('allows remote images only when tracking protection is off', () => {
    expect(csp(buildMailFrameSrcdoc('', true))).toContain('img-src * data: blob:');
  });

  it('ships a syntactically valid bridge script', () => {
    const srcdoc = buildMailFrameSrcdoc('', false);
    const script = srcdoc.match(/<script nonce="[^"]+">([\s\S]*?)<\/script>/)?.[1];
    expect(script).toBeTruthy();
    expect(() => new Function(script ?? '')).not.toThrow();
    expect(script).toContain("source: 'kestrel-mail'");
  });
});
