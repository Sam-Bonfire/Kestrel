import { fetch as tauriFetch } from '@tauri-apps/plugin-http';

export interface LinkPreviewData {
  url: string;
  domain: string;
  title?: string;
  description?: string;
  imageUrl?: string;
  faviconUrl?: string;
}

const CACHE_KEY = 'kestrel:link_previews';
let memoryCache: Record<string, LinkPreviewData> | null = null;

function getCache(): Record<string, LinkPreviewData> {
  if (memoryCache) return memoryCache;
  try {
    const raw = sessionStorage.getItem(CACHE_KEY);
    if (raw) {
      memoryCache = JSON.parse(raw);
      return memoryCache!;
    }
  } catch (e) {
    // Ignore cache load failure
  }
  memoryCache = {};
  return memoryCache;
}

function saveCache(cache: Record<string, LinkPreviewData>) {
  memoryCache = cache;
  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(cache));
  } catch (e) {
    // Ignore cache save failure
  }
}

async function doFetch(url: string, timeoutMs: number): Promise<string> {
  const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__;

  if (isTauri) {
    const res = await tauriFetch(url, {
      method: 'GET',
      connectTimeout: timeoutMs,
    });
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    return res.text();
  } else {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const res = await window.fetch(url, {
        method: 'GET',
        signal: controller.signal,
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      return await res.text();
    } finally {
      clearTimeout(timeoutId);
    }
  }
}

function parseHtml(html: string, originalUrl: string, domain: string): LinkPreviewData {
  const result: LinkPreviewData = { url: originalUrl, domain };

  // Note: Using DOMParser in browser environment. Tests might need to mock this if running in node without jsdom,
  // but standard vitest with jsdom provides DOMParser.
  if (typeof DOMParser !== 'undefined') {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');

    // Title
    const ogTitle = doc.querySelector('meta[property="og:title"]')?.getAttribute('content');
    const title = doc.querySelector('title')?.textContent;
    result.title = ogTitle || title || undefined;

    // Description
    const ogDesc = doc.querySelector('meta[property="og:description"]')?.getAttribute('content');
    const metaDesc = doc.querySelector('meta[name="description"]')?.getAttribute('content');
    result.description = ogDesc || metaDesc || undefined;

    // Image
    const ogImage = doc.querySelector('meta[property="og:image"]')?.getAttribute('content');
    if (ogImage) {
      result.imageUrl = new URL(ogImage, originalUrl).toString();
    }

    // Favicon
    let faviconUrl = undefined;
    const iconLinks = Array.from(doc.querySelectorAll('link[rel="icon"], link[rel="shortcut icon"]'));
    if (iconLinks.length > 0) {
      const href = iconLinks[0].getAttribute('href');
      if (href) {
        faviconUrl = new URL(href, originalUrl).toString();
      }
    }

    if (!faviconUrl) {
      try {
          const urlObj = new URL(originalUrl);
          faviconUrl = `${urlObj.protocol}//${urlObj.host}/favicon.ico`;
      } catch (e) {
          // Ignore
      }
    }
    result.faviconUrl = faviconUrl;

  } else {
    // Basic regex fallback if DOMParser is somehow not available
    const ogTitleMatch = html.match(/<meta[^>]*property="og:title"[^>]*content="([^"]+)"[^>]*>/i) || html.match(/<meta[^>]*content="([^"]+)"[^>]*property="og:title"[^>]*>/i);
    const titleMatch = html.match(/<title[^>]*>([^<]+)<\/title>/i);
    result.title = ogTitleMatch?.[1] || titleMatch?.[1];

    const ogDescMatch = html.match(/<meta[^>]*property="og:description"[^>]*content="([^"]+)"[^>]*>/i) || html.match(/<meta[^>]*content="([^"]+)"[^>]*property="og:description"[^>]*>/i);
    const metaDescMatch = html.match(/<meta[^>]*name="description"[^>]*content="([^"]+)"[^>]*>/i) || html.match(/<meta[^>]*content="([^"]+)"[^>]*name="description"[^>]*>/i);
    result.description = ogDescMatch?.[1] || metaDescMatch?.[1];

    const ogImageMatch = html.match(/<meta[^>]*property="og:image"[^>]*content="([^"]+)"[^>]*>/i) || html.match(/<meta[^>]*content="([^"]+)"[^>]*property="og:image"[^>]*>/i);
    if (ogImageMatch?.[1]) {
      try {
        result.imageUrl = new URL(ogImageMatch[1], originalUrl).toString();
      } catch (e) {}
    }

    const iconMatch = html.match(/<link[^>]*rel="(?:shortcut )?icon"[^>]*href="([^"]+)"[^>]*>/i) || html.match(/<link[^>]*href="([^"]+)"[^>]*rel="(?:shortcut )?icon"[^>]*>/i);
    if (iconMatch?.[1]) {
      try {
        result.faviconUrl = new URL(iconMatch[1], originalUrl).toString();
      } catch(e) {}
    }
    if (!result.faviconUrl) {
        try {
            const urlObj = new URL(originalUrl);
            result.faviconUrl = `${urlObj.protocol}//${urlObj.host}/favicon.ico`;
        } catch(e) {}
    }
  }

  return result;
}

export async function fetchLinkPreview(url: string): Promise<LinkPreviewData | null> {
  try {
    const urlObj = new URL(url);
    const domain = urlObj.hostname;

    const cache = getCache();
    if (cache[url]) {
      return cache[url];
    }

    // Try fetching
    const html = await doFetch(url, 4000);
    const data = parseHtml(html, url, domain);

    cache[url] = data;
    saveCache(cache);

    return data;

  } catch (error) {
    console.warn(`Failed to fetch link preview for ${url}`, error);
    // If failed, we still cache a basic fallback to avoid re-fetching on every render
    try {
      const urlObj = new URL(url);
      const fallback: LinkPreviewData = {
          url,
          domain: urlObj.hostname,
          faviconUrl: `${urlObj.protocol}//${urlObj.host}/favicon.ico`
      };
      const cache = getCache();
      cache[url] = fallback;
      saveCache(cache);
      return fallback;
    } catch(e) {
        return null;
    }
  }
}

// For testing purposes
export function clearLinkPreviewCache() {
  memoryCache = null;
  try {
    sessionStorage.removeItem(CACHE_KEY);
  } catch (e) {}
}
