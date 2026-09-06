import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { fetchLinkPreview, clearLinkPreviewCache } from './linkPreview.js';

describe('linkPreview', () => {
  let originalWindow: any;

  beforeEach(() => {
    clearLinkPreviewCache();
    vi.stubGlobal('fetch', vi.fn());
    vi.stubGlobal('sessionStorage', {
      getItem: vi.fn(),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    });

    originalWindow = global.window;

    global.window = {
        fetch: globalThis.fetch
    } as any;
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    global.window = originalWindow;
  });

  it('fetches and parses OpenGraph metadata', async () => {
    const mockHtml = `
      <html>
        <head>
          <meta property="og:title" content="Test Title">
          <meta property="og:description" content="Test Description">
          <meta property="og:image" content="https://example.com/image.png">
          <link rel="icon" href="/favicon.png">
        </head>
        <body></body>
      </html>
    `;
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      text: async () => mockHtml,
    } as any);

    const result = await fetchLinkPreview('https://example.com/page');

    expect(result).toEqual({
      url: 'https://example.com/page',
      domain: 'example.com',
      title: 'Test Title',
      description: 'Test Description',
      imageUrl: 'https://example.com/image.png',
      faviconUrl: 'https://example.com/favicon.png',
    });
  });

  it('falls back to standard title and meta description', async () => {
    const mockHtml = `
      <html>
        <head>
          <title>Fallback Title</title>
          <meta name="description" content="Fallback Description">
        </head>
        <body></body>
      </html>
    `;
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      text: async () => mockHtml,
    } as any);

    const result = await fetchLinkPreview('https://example.com/page2');

    expect(result).toMatchObject({
      title: 'Fallback Title',
      description: 'Fallback Description',
      faviconUrl: 'https://example.com/favicon.ico', // Default fallback
    });
  });

  it('caches the result to avoid redundant fetches', async () => {
    const mockHtml = `<html><head><title>Cached</title></head></html>`;
    vi.mocked(globalThis.fetch).mockResolvedValueOnce({
      ok: true,
      text: async () => mockHtml,
    } as any);

    const result1 = await fetchLinkPreview('https://example.com/cache-test');
    expect(result1?.title).toBe('Cached');
    expect(globalThis.fetch).toHaveBeenCalledTimes(1);

    const result2 = await fetchLinkPreview('https://example.com/cache-test');
    expect(result2?.title).toBe('Cached');
    expect(globalThis.fetch).toHaveBeenCalledTimes(1); // Still 1
  });

  it('returns a fallback domain and favicon if fetch fails', async () => {
    vi.mocked(globalThis.fetch).mockRejectedValueOnce(new Error('Network error'));

    const result = await fetchLinkPreview('https://fail.com/test');

    expect(result).toEqual({
      url: 'https://fail.com/test',
      domain: 'fail.com',
      faviconUrl: 'https://fail.com/favicon.ico',
    });
    // Ensure failure is also cached so it doesn't spam network
    expect(globalThis.fetch).toHaveBeenCalledTimes(1);
    await fetchLinkPreview('https://fail.com/test');
    expect(globalThis.fetch).toHaveBeenCalledTimes(1);
  });
});
