export type Bucket = 'inbox' | 'paper' | 'feed';

export interface RoutableMail {
  senderEmail?: string | null;
  subject?: string | null;
  snippet?: string | null;
  labels?: string[];
}

const RECEIPT_PATTERNS = [
  'receipt',
  'order confirmation',
  'order confirmed',
  'your order',
  'invoice',
  'billing statement',
  'payment received',
  'payment confirmation',
  'transaction',
  'booking confirmation',
  'reservation confirmed',
  'ticket confirmation',
  'shipping confirmation',
  'shipped',
  'delivery update',
  'statement is ready',
  'monthly statement',
];

const RECEIPT_SENDERS = ['no-reply', 'noreply', 'donotreply', 'billing', 'receipts', 'orders', 'support'];

function worded(hay: string, word: string): boolean {
  return new RegExp(`\\b${word}s?\\b`).test(hay);
}

/** Keyword + sender-pattern pass. No network, no models. */
export function isTransactional(mail: RoutableMail): boolean {
  const hay = `${mail.subject ?? ''}\n${mail.snippet ?? ''}`.toLowerCase();
  if (RECEIPT_PATTERNS.some((p) => hay.includes(p))) return true;
  const from = (mail.senderEmail ?? '').toLowerCase();
  const local = from.split('@')[0] ?? '';
  const knownSender = RECEIPT_SENDERS.some((s) => local === s || local.startsWith(`${s}-`) || local.startsWith(`${s}.`));
  if (knownSender) {
    return (
      worded(hay, 'order') ||
      worded(hay, 'receipt') ||
      worded(hay, 'invoice') ||
      worded(hay, 'payment') ||
      worded(hay, 'booking') ||
      hay.includes('$') ||
      /invoice\s*#?\s*\d|tracking\s*#?\s*\w|order\s*#?\s*\w/i.test(
        `${mail.subject ?? ''}\n${mail.snippet ?? ''}`
      )
    );
  }
  return false;
}

export type PinMap = Record<string, Bucket>;

/**
 * Fixed routing order: explicit user label pin wins, then per-sender
 * override, then the keyword classifier. Returns the forced bucket or
 * null when default view routing applies.
 */
export function resolveBucket(
  mail: RoutableMail,
  labelPins: PinMap,
  senderPins: PinMap,
): Bucket | null {
  for (const label of mail.labels ?? []) {
    const pin = labelPins[label];
    if (pin) return pin;
  }
  const sender = (mail.senderEmail ?? '').trim().toLowerCase();
  if (sender && senderPins[sender]) return senderPins[sender];
  if (isTransactional(mail)) return 'paper';
  return null;
}
