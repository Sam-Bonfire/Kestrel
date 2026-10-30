import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import CenterPeek from './CenterPeek.svelte';
import { set } from 'idb-keyval';
import type { ComponentProps } from 'svelte';
import { tick } from 'svelte';

type EmailProp = NonNullable<ComponentProps<typeof CenterPeek>['email']>;

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

vi.mock('$app/environment', () => ({
  browser: true,
}));

vi.mock('idb-keyval', () => ({
  get: vi.fn().mockResolvedValue(null),
  set: vi.fn().mockResolvedValue(undefined),
}));

describe('CenterPeek - Iframe Hardening', () => {
  it('should block javascript execution via XSS vectors in the iframe srcdoc', async () => {
    // This is tested by injecting a script tag into the HTML and verifying the srcdoc CSP
    // and DOMPurify prevent it.
    const mockEmail: EmailProp = {
      id: '123',
      sender: 'Test',
      senderEmail: 'test@example.com',
      to: 'me@example.com',
      subject: 'Test XSS',
      body: '<script>window.parent.postMessage("xss")</script><img src="javascript:alert(1)" onerror="alert(1)">',
      timestamp: '2026-01-01T00:00:00Z',
      isUnread: false,
      isStarred: false,
      isArchived: false,
      isTrash: false,
      labels: [],
    };

    const { container } = render(CenterPeek, {
      props: {
        email: mockEmail,
        onArchive: vi.fn(),
        onDelete: vi.fn(),
        onToggleStar: vi.fn(),
        onClipSelection: vi.fn(),
        onClose: vi.fn(),
      },
    });

    await tick();

    const iframe = container.querySelector('iframe');
    expect(iframe).toBeTruthy();

    // Sandbox must not grant same-origin: the frame can never read the app DOM.
    const sandbox = iframe?.getAttribute('sandbox') || '';
    expect(sandbox).not.toContain('allow-same-origin');

    // SRC doc should have the restrictive CSP
    const srcdoc = iframe?.getAttribute('srcdoc') || '';
    expect(srcdoc).toContain('Content-Security-Policy');
    expect(srcdoc).toContain("default-src 'none'");
    // Scripts are limited to the nonced bridge, so injected inline script dies.
    expect(srcdoc).toContain("script-src 'nonce-");

    // DOMPurify should strip script tags
    expect(srcdoc).not.toContain('window.parent.postMessage("xss")');
    expect(srcdoc).not.toContain('alert(1)');
    expect(srcdoc).not.toContain('onerror=');
  });

  it('ignores checklist messages that do not come from the body frame', async () => {
    const mockEmail: EmailProp = {
      id: '456',
      sender: 'Test',
      senderEmail: 'test@example.com',
      to: 'me@example.com',
      subject: 'Bridge spoofing',
      body: '- [ ] Task',
      timestamp: '2026-01-01T00:00:00Z',
      isUnread: false,
      isStarred: false,
      isArchived: false,
      isTrash: false,
      labels: [],
    };

    render(CenterPeek, { props: { email: mockEmail } });
    await tick();

    vi.mocked(set).mockClear();
    // No `source`, so this never came from the iframe's contentWindow.
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { source: 'kestrel-mail', type: 'checklist', data: { taskId: 'task-1', checked: true } },
      })
    );
    await tick();

    expect(set).not.toHaveBeenCalled();
  });
});
