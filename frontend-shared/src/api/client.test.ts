import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  API_BASE,
  ApiError,
  getHealth,
  register,
  createToken,
  getMessages,
  getMessage,
  sendMessage,
  createEvent,
  updateEvent,
  deleteEvent,
  getSettings,
  updateSettings,
  searchMessages,
} from './client.js';
import type {
  CreateEventRequest,
  SendMessageRequest,
  SettingsPayload,
} from './generated/types.js';

describe('API Client & Contract Validation', () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it('getHealth sends GET request and parses response', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({ status: 'ok' }),
    });

    const res = await getHealth();
    expect(res).toEqual({ status: 'ok' });
    expect(globalThis.fetch).toHaveBeenCalledWith(
      `${API_BASE}/health`,
      expect.objectContaining({
        method: 'GET',
        headers: expect.objectContaining({
          'Content-Type': 'application/json',
        }),
      })
    );
  });

  it('register sends POST /auth/register with credentials', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 201,
      json: async () => ({ user_id: 'usr-12345' }),
    });

    const res = await register('testuser@kestrel.dev', 'SecurePass123!');
    expect(res).toEqual({ user_id: 'usr-12345' });
    expect(globalThis.fetch).toHaveBeenCalledWith(
      `${API_BASE}/auth/register`,
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          email: 'testuser@kestrel.dev',
          password: 'SecurePass123!',
        }),
      })
    );
  });

  it('createToken injects credentials and returns JWT payload', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({ token: 'jwt.token.val', user_id: 'usr-12345' }),
    });

    const res = await createToken('testuser@kestrel.dev', 'SecurePass123!');
    expect(res.token).toBe('jwt.token.val');
    expect(res.user_id).toBe('usr-12345');
  });

  it('getMessages constructs query params correctly with filters', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        messages: [],
        total: 0,
      }),
    });

    await getMessages('mock-jwt', 2, 25);

    expect(globalThis.fetch).toHaveBeenCalledWith(
      expect.stringContaining('/messages?page=2&per_page=25'),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer mock-jwt',
        }),
      })
    );
  });

  it('sendMessage serializes Specta SendMessageRequest contract', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({ id: 'local-sent-uuid' }),
    });

    const payload: SendMessageRequest = {
      account_id: '00000000-0000-0000-0000-000000000001',
      to: ['recipient@kestrel.dev'],
      cc: null,
      bcc: null,
      subject: 'Contract Test',
      body_html: '<p>Hello Specta</p>',
      attachments: null,
    };

    const res = await sendMessage(payload, 'mock-jwt');
    expect(res).toEqual({ id: 'local-sent-uuid' });
    expect(globalThis.fetch).toHaveBeenCalledWith(
      `${API_BASE}/messages/send`,
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          Authorization: 'Bearer mock-jwt',
        }),
        body: JSON.stringify(payload),
      })
    );
  });

  it('createEvent serializes Specta CreateEventRequest payload', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 201,
      json: async () => ({
        id: 'ev-123',
        calendar_id: 'cal-123',
        title: 'New Meeting',
        start_time: 1720000000,
        end_time: 1720003600,
        created_at: 1720000000,
      }),
    });

    const payload: CreateEventRequest = {
      calendar_id: '00000000-0000-0000-0000-000000000002',
      title: 'New Meeting',
      description: 'Discuss Roadmap',
      location: 'Virtual',
      start_time: 1720000000,
      end_time: 1720003600,
      is_all_day: false,
      recurrence_rules: null,
      attendees: null,
    };

    const res = await createEvent(payload, 'mock-jwt');
    expect(res.title).toBe('New Meeting');
    expect(globalThis.fetch).toHaveBeenCalledWith(
      `${API_BASE}/events`,
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify(payload),
      })
    );
  });

  it('updateSettings sends partial SettingsPayload to PUT /api/settings', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
        mailDenseMode: true,
        syncInterval: 300,
      }),
    });

    const partialSettings: Partial<SettingsPayload> = {
      mailDenseMode: true,
      syncInterval: 300,
    };

    const res = await updateSettings(partialSettings, 'mock-jwt');
    expect(res.mailDenseMode).toBe(true);
    expect(globalThis.fetch).toHaveBeenCalledWith(
      'http://localhost:8080/api/settings',
      expect.objectContaining({
        method: 'PUT',
        body: JSON.stringify(partialSettings),
      })
    );
  });

  it('ApiError extracts error message and status code on failure', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: 'Not Found',
      json: async () => ({ message: 'Message not found' }),
    });

    await expect(getMessage('nonexistent-id', 'mock-jwt')).rejects.toThrow(ApiError);
  });
});
