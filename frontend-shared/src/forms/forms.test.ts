import { afterEach, describe, expect, it, vi } from 'vitest';
import { createFormsClient } from './index.js';

const submission = {
  app: 'landing' as const,
  form: 'interest',
  fields: { email: 'someone@example.com' },
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('createFormsClient', () => {
  it('is disabled and never calls fetch when the endpoint is unset', async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    const client = createFormsClient({ endpoint: '' });
    const result = await client.submit(submission);

    expect(client.enabled).toBe(false);
    expect(result).toEqual({ status: 'disabled' });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('posts the contract payload with an empty honeypot', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 202 });
    vi.stubGlobal('fetch', fetchMock);

    const client = createFormsClient({ endpoint: 'https://forms.example.com/ingest' });
    const result = await client.submit({
      ...submission,
      meta: { turnstile: 'token-1' },
    });

    expect(result).toEqual({ status: 'sent' });
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('https://forms.example.com/ingest');
    expect(init.method).toBe('POST');

    const body = JSON.parse(String(init.body)) as {
      app: string;
      form: string;
      fields: Record<string, string>;
      meta: { turnstile?: string; sentAt?: string };
    };
    expect(body.app).toBe('landing');
    expect(body.form).toBe('interest');
    expect(body.fields).toEqual({ _hp: '', email: 'someone@example.com' });
    expect(body.meta.turnstile).toBe('token-1');
    expect(Date.parse(body.meta.sentAt ?? '')).not.toBeNaN();
  });

  it('sends the bearer token only when configured', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 200 });
    vi.stubGlobal('fetch', fetchMock);

    await createFormsClient({
      endpoint: 'https://forms.example.com/ingest',
      token: 'abc123',
    }).submit(submission);
    await createFormsClient({ endpoint: 'https://forms.example.com/ingest' }).submit(submission);

    const withToken = fetchMock.mock.calls[0]?.[1] as RequestInit;
    const withoutToken = fetchMock.mock.calls[1]?.[1] as RequestInit;
    expect((withToken.headers as Record<string, string>).authorization).toBe('Bearer abc123');
    expect((withoutToken.headers as Record<string, string>).authorization).toBeUndefined();
  });

  it('keeps a filled honeypot intact so the receiver can drop it', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 200 });
    vi.stubGlobal('fetch', fetchMock);

    await createFormsClient({ endpoint: 'https://forms.example.com/ingest' }).submit({
      ...submission,
      fields: { email: 'bot@example.com', _hp: 'filled-by-bot' },
    });

    const body = JSON.parse(String((fetchMock.mock.calls[0]?.[1] as RequestInit).body)) as {
      fields: Record<string, string>;
    };
    expect(body.fields._hp).toBe('filled-by-bot');
  });

  it('reports rejections instead of throwing', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 400 }));
    const rejected = await createFormsClient({ endpoint: 'https://forms.example.com/ingest' }).submit(
      submission,
    );
    expect(rejected).toEqual({ status: 'rejected', error: 'Forms endpoint returned 400' });

    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));
    const offline = await createFormsClient({ endpoint: 'https://forms.example.com/ingest' }).submit(
      submission,
    );
    expect(offline).toEqual({ status: 'rejected', error: 'offline' });
  });
});
