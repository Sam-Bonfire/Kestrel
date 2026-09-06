import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const printCss = readFileSync(join(import.meta.dirname, 'print.css'), 'utf8');

describe('print stylesheet', () => {
  it('forces light paper output inside @media print', () => {
    expect(printCss).toContain('@media print');
    expect(printCss).toContain('background: white !important');
  });

  it('hides app chrome marked .no-print and sets paper margins', () => {
    expect(printCss).toContain('.no-print');
    expect(printCss).toContain('display: none !important');
    expect(printCss).toContain('@page');
    expect(printCss).toContain('margin: 1.5cm');
  });

  it('reveals .print-only content and avoids breaking rows', () => {
    expect(printCss).toContain('.print-only');
    expect(printCss).toContain('display: block !important');
    expect(printCss).toContain('break-inside: avoid');
  });
});
