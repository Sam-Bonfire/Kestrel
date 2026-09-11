export interface ScreenableMessage {
  id: string;
  senderEmail: string;
  sender?: string;
  subject?: string;
  timestamp?: string;
}

export interface ScreenedSender {
  email: string;
  name: string;
  messageId: string;
  subject: string;
  timestamp: string;
}

const SCREENED_KEY = 'kestrel:screener:screened';

function normalize(email: string): string {
  return email.trim().toLowerCase();
}

export function loadScreened(): Set<string> {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(SCREENED_KEY);
      const parsed = raw !== null ? JSON.parse(raw) : [];
      return new Set(Array.isArray(parsed) ? parsed : []);
    }
    return new Set();
  } catch {
    return new Set();
  }
}

function saveScreened(emails: Set<string>) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(SCREENED_KEY, JSON.stringify([...emails]));
    }
  } catch {
    // Non-fatal
  }
}

function key(email: string, scope: string): string {
  const normalized = normalize(email);
  return scope ? `${scope}:${normalized}` : normalized;
}

/** Mark a sender as reviewed so they leave the screener queue. */
export function approveSender(email: string, screened?: Set<string>, scope = ''): Set<string> {
  const next = new Set(screened ?? loadScreened());
  const k = key(email, scope);
  if (k) next.add(k);
  saveScreened(next);
  return next;
}

/**
 * First-time senders: exactly one message in the loaded list and not
 * yet reviewed. Newest first.
 *
 * ponytail: heuristic over the loaded (30-day) window; a regular sender
 * with a single in-window message also qualifies. Use a server-side
 * sender count when one exists.
 */
export function firstTimeSenders(
  messages: ScreenableMessage[],
  screened?: Set<string>,
  scope = ''
): ScreenedSender[] {
  const reviewed = screened ?? loadScreened();
  const counts = new Map<string, number>();
  for (const m of messages) {
    const k = key(m.senderEmail ?? '', scope);
    if (!k) continue;
    counts.set(k, (counts.get(k) ?? 0) + 1);
  }
  return messages
    .filter((m) => {
      const k = key(m.senderEmail ?? '', scope);
      return k && counts.get(k) === 1 && !reviewed.has(k);
    })
    .sort((a, b) => (b.timestamp ?? '').localeCompare(a.timestamp ?? ''))
    .map((m) => ({
      email: normalize(m.senderEmail),
      name: m.sender ?? m.senderEmail,
      messageId: m.id,
      subject: m.subject ?? '(no subject)',
      timestamp: m.timestamp ?? '',
    }));
}
