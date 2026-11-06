import 'fake-indexeddb/auto';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import EventPeekPanel from './EventPeekPanel.svelte';
import * as api from '@kestrel/shared/api';

vi.mock('@kestrel/shared/api', () => ({
  dismissEventConflict: vi.fn().mockResolvedValue(undefined),
  restoreRevision: vi.fn().mockResolvedValue(undefined),
  getEvent: vi.fn(),
  updateEvent: vi.fn(),
  deleteEvent: vi.fn(),
}));

describe('EventPeekPanel conflict banner', () => {
  const mockEvent = {
    id: 'evt-123',
    title: 'Conflict Event',
    date: '2024-01-01',
    startTime: '10:00 AM',
    endTime: '11:00 AM',
    color: '#ff0000',
    conflict: {
      revision_id: 'rev-789',
      revision_number: 2,
      detected_at: 1696161600,
      reason: 'remote-changed-while-local-modified'
    }
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the conflict banner if conflict exists', async () => {
    render(EventPeekPanel, { props: { event: mockEvent, onClose: () => {} } });
    await waitFor(() => {
      expect(screen.getByText(/This event changed on the server/)).toBeInTheDocument();
    });
    expect(screen.getByText('Keep mine')).toBeInTheDocument();
    expect(screen.getByText('Use server version')).toBeInTheDocument();
  });
});
