export type KestrelDeepLink =
  | { app: 'mail'; kind: 'thread'; id: string }
  | { app: 'calendar'; kind: 'event'; id: string };

/**
 * Parse OS deep links into specific threads/events.
 * Only schemes the OS actually routes are accepted: `kestrel://`
 * (owned by the mail app) and `kestrel-calendar://` (owned by the
 * calendar app). Returns null for anything else (e.g. OAuth
 * callbacks), which other handlers own.
 */
export function parseKestrelDeepLink(url: string): KestrelDeepLink | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }
  const scheme = parsed.protocol.replace(/:$/, '');
  // For custom schemes the first segment is the hostname
  // (kestrel://mail/thread/t-1 -> host 'mail', path '/thread/t-1').
  const segments = [parsed.hostname, ...parsed.pathname.split('/')].filter(Boolean);

  if (scheme === 'kestrel' && segments.length === 3) {
    const [app, kind, id] = segments;
    if (app === 'mail' && kind === 'thread' && id) return { app, kind, id: decodeURIComponent(id) };
    return null;
  }
  if (scheme === 'kestrel-calendar' && segments.length === 2) {
    const [kind, id] = segments;
    if (kind === 'event' && id) return { app: 'calendar', kind, id: decodeURIComponent(id) };
    return null;
  }
  return null;
}

export function buildThreadDeepLink(id: string): string {
  return `kestrel://mail/thread/${encodeURIComponent(id)}`;
}

export function buildEventDeepLink(id: string): string {
  return `kestrel-calendar://event/${encodeURIComponent(id)}`;
}
