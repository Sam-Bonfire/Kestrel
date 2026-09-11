export interface TriageMessage {
  id: string;
  isUnread: boolean;
  isStarred?: boolean;
  isArchived?: boolean;
  isTrash?: boolean;
  category?: string;
  /** ISO timestamp */
  timestamp?: string;
  senderEmail?: string;
  subject?: string;
  snippet?: string;
  labels?: string[];
}

export interface TriageOptions {
  olderThanDays?: number;
  categories?: string[];
}

const DEFAULT_CATEGORIES = ['Promotions', 'Social'];
const DEFAULT_OLDER_THAN_DAYS = 30;

// Fallback promo signal when there is no usable category: 'Primary' is the
// default bucket (and the only one before the split-inbox classifier runs),
// so only high-precision bulk-mail markers qualify here. K-376 owns taxonomy.
const PROMO_FALLBACK_KEYWORDS = ['unsubscribe', 'newsletter'];

const LAST_RUN_KEY = 'kestrel:triage:last-run';

function toISODate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

/** Once per calendar day. */
export function shouldRunTriage(now: Date = new Date()): boolean {
  try {
    if (typeof localStorage === 'undefined') return false;
    return localStorage.getItem(LAST_RUN_KEY) !== toISODate(now);
  } catch {
    return false;
  }
}

export function markTriageRun(now: Date = new Date()) {
  try {
    if (typeof localStorage !== 'undefined') localStorage.setItem(LAST_RUN_KEY, toISODate(now));
  } catch {
    // Non-fatal
  }
}

/**
 * Read, unstarred, low-value mail older than the threshold.
 *
 * ponytail: fixed 30-day / Promotions+Social default, no per-sender
 * learning. Add adaptive thresholds when users ask for them.
 */
export function triageCandidates(
  messages: TriageMessage[],
  now: Date = new Date(),
  opts: TriageOptions = {}
): string[] {
  const olderThanDays = opts.olderThanDays ?? DEFAULT_OLDER_THAN_DAYS;
  const categories = opts.categories ?? DEFAULT_CATEGORIES;
  const cutoff = now.getTime() - olderThanDays * 24 * 60 * 60 * 1000;
  return messages
    .filter((m) => {
      if (m.isUnread || m.isStarred || m.isArchived || m.isTrash) return false;
      if (!m.timestamp || new Date(m.timestamp).getTime() >= cutoff) return false;
      if (m.category && m.category !== 'Primary') return categories.includes(m.category);
      const haystack = `${m.subject ?? ''}\n${m.snippet ?? ''}`.toLowerCase();
      return PROMO_FALLBACK_KEYWORDS.some((k) => haystack.includes(k));
    })
    .map((m) => m.id)
    .slice(0, 50);
}
