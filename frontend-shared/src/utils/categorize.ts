export type EmailCategory = 'Primary' | 'Updates' | 'Social' | 'Promotions' | 'Forums';

export interface Categorizable {
  senderEmail?: string;
  subject?: string;
  snippet?: string;
  labels?: string[];
}

const SOCIAL_DOMAINS = [
  'facebook.com', 'twitter.com', 'x.com', 'instagram.com', 'linkedin.com',
  'tiktok.com', 'reddit.com', 'youtube.com', 'pinterest.com', 'snapchat.com',
];

const UPDATES_DOMAINS = [
  'github.com', 'gitlab.com', 'linear.app', 'asana.com', 'trello.com',
  'circleci.com', 'vercel.com', 'netlify.com',
];

// Matched against dot-separated domain labels, never substrings.
const UPDATES_TOKENS = ['jira', 'datadog', 'travis'];

const TRANSACTIONAL_KEYWORDS = [
  'receipt', 'invoice', 'order confirmation', 'shipping', 'tracking number',
  'calendar invite', 'invitation:', 'rsvp', 'meeting notes',
];

const FORUM_KEYWORDS = ['mailing list', 'google groups', 'discourse'];

const PROMO_KEYWORDS = [
  'unsubscribe', 'sale', 'deal', 'discount', 'offer', '% off', 'coupon', 'newsletter',
];

const SENT_LABELS = ['sent', 'outbox', 'draft'];

function domainOf(email: string): string {
  return (email.trim().toLowerCase().split('@')[1] ?? '');
}

function matchesDomain(domain: string, candidates: string[]): boolean {
  return candidates.some((d) => domain === d || domain.endsWith(`.${d}`));
}

function matchesToken(domain: string, tokens: string[]): boolean {
  return domain.split('.').some((label) => tokens.includes(label));
}

/**
 * Deterministic rules-based classifier for split-inbox tabs.
 * Precedence: Social > Updates > Forums > Promotions > Primary.
 *
 * ponytail: naive keyword/domain heuristic, no ML. Upgrade to a trained
 * model or server-side user rules when misclassification is reported.
 */
export function categorizeEmail(message: Categorizable): EmailCategory {
  const domain = domainOf(message.senderEmail ?? '');
  const subject = (message.subject ?? '').toLowerCase();
  const snippet = (message.snippet ?? '').toLowerCase();
  const labels = (message.labels ?? []).map((l) => l.toLowerCase());
  const haystack = `${subject}\n${snippet}`;

  if (labels.some((l) => SENT_LABELS.includes(l))) return 'Primary';
  if (matchesDomain(domain, SOCIAL_DOMAINS)) return 'Social';
  if (
    matchesDomain(domain, UPDATES_DOMAINS) ||
    matchesToken(domain, UPDATES_TOKENS) ||
    labels.some((l) => l.includes('notification') || l.includes('ci')) ||
    TRANSACTIONAL_KEYWORDS.some((k) => haystack.includes(k))
  ) {
    return 'Updates';
  }
  if (FORUM_KEYWORDS.some((k) => haystack.includes(k))) return 'Forums';
  if (PROMO_KEYWORDS.some((k) => haystack.includes(k))) return 'Promotions';
  return 'Primary';
}
