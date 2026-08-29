import { test, expect } from 'vitest';
import { parseChecklists } from './markdownParser';

test('parses basic checklists', () => {
    const res1 = parseChecklists('- [ ] Task');
    expect(res1).toContain('<input type="checkbox"');
    expect(res1).not.toContain('checked');

    const res2 = parseChecklists('- [x] Completed');
    expect(res2).toContain('<input type="checkbox"');
    expect(res2).toContain('checked');

    const res3 = parseChecklists('* [ ] Another task');
    expect(res3).toContain('<input type="checkbox"');

    // Make sure we handle multiline text and retain standard styling
    const multiline = `
- [ ] First
- [x] Second
    `;
    const resMulti = parseChecklists(multiline);
    expect(resMulti).toContain('First');
    expect(resMulti).toContain('Second');
});
