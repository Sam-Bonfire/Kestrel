import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CenterPeek from './CenterPeek.svelte';
import { tick } from 'svelte';

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
    const mockEmail = {
      id: '123',
      subject: 'Test XSS',
      body: '<script>window.parent.postMessage("xss")</script><img src="javascript:alert(1)" onerror="alert(1)">',
      bodyText: 'fallback',
      senderName: 'Test',
      senderEmail: 'test@example.com',
      timestamp: Date.now(),
      recipients: [],
      isRead: true,
    };

    const { container } = render(CenterPeek, {
      props: {
        email: mockEmail as any,
        isArchived: false,
        onArchive: vi.fn(),
        onDelete: vi.fn(),
        onRestore: vi.fn(),
        onMarkUnread: vi.fn(),
        onToggleStar: vi.fn(),
        onSetAside: vi.fn(),
        onAddClip: vi.fn(),
        onRemoveClip: vi.fn(),
        onClipSelection: vi.fn(),
        onClose: vi.fn(),
      }
    });

    await tick();

    const iframe = container.querySelector('iframe');
    expect(iframe).toBeTruthy();

    // Sandbox should lack allow-scripts and allow-same-origin
    const sandbox = iframe?.getAttribute('sandbox') || '';
    expect(sandbox).not.toContain('allow-scripts');
    expect(sandbox).not.toContain('allow-same-origin');

    // SRC doc should have the restrictive CSP
    const srcdoc = iframe?.getAttribute('srcdoc') || '';
    expect(srcdoc).toContain('Content-Security-Policy');
    expect(srcdoc).toContain("default-src 'none'");

    // DOMPurify should strip script tags
    expect(srcdoc).not.toContain('<script>');
    expect(srcdoc).not.toContain('onerror=');
  });
});
