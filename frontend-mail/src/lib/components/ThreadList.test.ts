import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import ThreadList from './ThreadList.svelte';
import { mailStore } from '../stores/mailStore.svelte.js';

vi.mock('@kestrel/shared', () => ({
  mailDenseMode: { subscribe: (fn: any) => { fn(false); return () => {}; } },
  labelCustomizations: { subscribe: (fn: any) => { fn({}); return () => {}; } },
  getLabelStyle: vi.fn(),
  Dropdown: vi.fn()
}));

const mockThreads = [
  {
    id: 't1',
    sender: 'Alice',
    senderEmail: 'alice@example.com',
    subject: 'Hello',
    snippet: 'Hi there',
    date: '2023-01-01',
    isUnread: true,
    isStarred: false,
    hasAttachment: false,
    labels: [],
    category: 'Primary'
  },
  {
    id: 't2',
    sender: 'Bob',
    senderEmail: 'bob@example.com',
    subject: 'Report',
    snippet: 'Attached report',
    date: '2023-01-02',
    isUnread: false,
    isStarred: true,
    hasAttachment: true,
    labels: [],
    category: 'Updates'
  }
];

describe('ThreadList', () => {
  beforeEach(() => {
    mailStore.setUnreadFilter(false);
  });

  it('renders threads properly', () => {
    const { getByText } = render(ThreadList, {
      props: {
        threads: mockThreads,
        currentView: 'inbox'
      }
    });

    expect(getByText('Hello')).toBeTruthy();
    expect(getByText('Report')).toBeTruthy();
  });

  it('toggles unread filter state', async () => {
    const { getByRole, getByText, queryByText, container } = render(ThreadList, {
      props: {
        threads: mockThreads,
        currentView: 'inbox'
      }
    });

    // Both threads rendered initially
    expect(getByText('Hello')).toBeTruthy();
    expect(getByText('Report')).toBeTruthy();

    // In testing-library we can find it by title instead if needed, or text.
    // The previous error was because the toolbar has two unread toggles possibly? Actually, just use title.
    const unreadButton = getByRole('button', { name: /Toggle Unread Filter/i });

    // Toggle via button
    await fireEvent.click(unreadButton);
    expect(mailStore.unreadFilterOnly).toBe(true);

    // After setting filter, Bob's thread (read) should be missing
    // wait for tick via simple delay since reactivity might take a microtask
    await new Promise(r => setTimeout(r, 0));
    expect(queryByText('Report')).toBeNull();
    expect(getByText('Hello')).toBeTruthy();
  });

  it('shows empty state when no unread messages match filter', async () => {
    mailStore.setUnreadFilter(true);

    const { getByText, getByRole } = render(ThreadList, {
      props: {
        threads: [], // Mock filteredList returning empty when no match
        currentView: 'inbox'
      }
    });

    await new Promise(r => setTimeout(r, 0));
    expect(getByText('No unread messages in this folder')).toBeTruthy();

    const showAllBtn = getByRole('button', { name: 'Show all messages' });
    await fireEvent.click(showAllBtn);

    expect(mailStore.unreadFilterOnly).toBe(false);
  });

  it('toggles unread filter via keyboard shortcut u', async () => {
    const { container } = render(ThreadList, {
      props: {
        threads: mockThreads,
        currentView: 'inbox'
      }
    });

    expect(mailStore.unreadFilterOnly).toBe(false);

    await fireEvent.keyDown(window, { key: 'u' });

    expect(mailStore.unreadFilterOnly).toBe(true);
  });
});
