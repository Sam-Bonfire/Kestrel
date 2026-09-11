export type InboxCategory = 'Primary' | 'Updates' | 'Social' | 'Promotions' | 'Forums';

export interface CategorizableEmail {
  senderEmail?: string;
  subject?: string;
  body?: string;
}

const SOCIAL_DOMAINS = ['facebook.com', 'twitter.com', 'x.com', 'instagram.com', 'linkedin.com'];
const UPDATES_DOMAINS = ['github.com', 'gitlab.com', 'linear.app', 'circleci.com'];
const FORUM_KEYWORDS = ['mailing list', 'google groups', 'discourse'];
const PROMO_KEYWORDS = ['unsubscribe', 'newsletter', 'sale', 'receipt', 'invoice'];

/**
 * Interim split-inbox classifier for the category tabs.
 * NOTE: delete in favor of the shared categorizeEmail engine (K-376)
 * once it merges — same rules, spoof-hardened, tested.
 */
export function inboxCategory(e: CategorizableEmail): InboxCategory {
  const domain = ((e.senderEmail ?? '').split('@')[1] ?? '').trim().toLowerCase();
  const hay = `${e.subject ?? ''}\n${(e.body ?? '').replace(/<[^>]*>?/gm, '')}`.toLowerCase();
  if (SOCIAL_DOMAINS.some((d) => domain === d || domain.endsWith(`.${d}`))) return 'Social';
  if (UPDATES_DOMAINS.some((d) => domain === d || domain.endsWith(`.${d}`))) return 'Updates';
  if (FORUM_KEYWORDS.some((k) => hay.includes(k))) return 'Forums';
  if (PROMO_KEYWORDS.some((k) => hay.includes(k))) return 'Promotions';
  return 'Primary';
}
