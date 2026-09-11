export interface NewsletterCandidate {
  senderEmail?: string;
  subject?: string;
  snippet?: string;
  labels?: string[];
}

const NEWSLETTER_MARKERS = [
  'unsubscribe',
  'newsletter',
  'view in browser',
  'view this email in your browser',
  'manage preferences',
  'email preferences',
  'daily digest',
  'weekly digest',
];

const NEWSLETTER_SENDERS = ['substack.com', 'beehiiv.com', 'convertkit.com', 'mailchimp.com', 'buttondown.email', 'ghost.io'];

function domainOf(email: string): string {
  return (email.trim().toLowerCase().split('@')[1] ?? '');
}

/**
 * Newsletter/bulk-bundle detector for the dedicated Feed view.
 *
 * ponytail: keyword + known-provider heuristic. Real List-Unsubscribe
 * header checks belong server-side when headers are stored.
 */
export function isNewsletter(message: NewsletterCandidate): boolean {
  const domain = domainOf(message.senderEmail ?? '');
  if (NEWSLETTER_SENDERS.some((d) => domain === d || domain.endsWith(`.${d}`))) return true;
  const labels = (message.labels ?? []).map((l) => l.toLowerCase());
  if (labels.some((l) => l.includes('newsletter') || l.includes('promotions'))) return true;
  const haystack = `${message.subject ?? ''}\n${message.snippet ?? ''}`.toLowerCase();
  return NEWSLETTER_MARKERS.some((k) => haystack.includes(k));
}
