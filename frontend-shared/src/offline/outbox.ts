const OUTBOX_KEY = 'kestrel:offline:outbox';

export type OutboxStatus = 'pending' | 'sending' | 'sent' | 'failed';

export interface OutboxItem {
  id: string;
  account_id: string;
  to: string;
  cc?: string;
  bcc?: string;
  subject: string;
  body_html?: string;
  body_text?: string;
  attachments_json?: string;
  status: OutboxStatus;
  created_at: string;
  retry_count: number;
  last_error?: string;
}

// ── Outbox CRUD ─────────────────────────────────────────────────

function readOutbox(): OutboxItem[] {
  try {
    const raw = localStorage.getItem(OUTBOX_KEY);
    return raw ? (JSON.parse(raw) as OutboxItem[]) : [];
  } catch {
    return [];
  }
}

function writeOutbox(outbox: OutboxItem[]): void {
  try {
    localStorage.setItem(OUTBOX_KEY, JSON.stringify(outbox));
  } catch {
    // Storage full — swallow.
  }
}

function generateId(): string {
  return `outbox_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

// ── Public API ──────────────────────────────────────────────────

/**
 * Enqueue a draft for later sending when back online.
 * Returns the id assigned to the outbox item.
 */
export function enqueueOutboxItem(
  item: Omit<OutboxItem, 'id' | 'status' | 'created_at' | 'retry_count'>
): string {
  const outboxItem: OutboxItem = {
    ...item,
    id: generateId(),
    status: 'pending',
    created_at: new Date().toISOString(),
    retry_count: 0,
  };

  const outbox = readOutbox();
  outbox.push(outboxItem);
  writeOutbox(outbox);
  return outboxItem.id;
}

/**
 * Return all outbox items ordered by creation time.
 */
export function getOutboxItems(): OutboxItem[] {
  const outbox = readOutbox();
  outbox.sort(
    (a, b) =>
      new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
  );
  return outbox;
}

/**
 * Update the status, retry count, or last error of an outbox item.
 */
export function updateOutboxItem(id: string, updates: Partial<OutboxItem>): void {
  const outbox = readOutbox();
  const index = outbox.findIndex((item) => item.id === id);
  if (index !== -1) {
    outbox[index] = { ...outbox[index], ...updates };
    writeOutbox(outbox);
  }
}

/**
 * Remove an item from the outbox after successful send or manual discard.
 */
export function removeOutboxItem(id: string): void {
  const outbox = readOutbox().filter((m) => m.id !== id);
  writeOutbox(outbox);
}

/**
 * Clear all outbox items.
 */
export function clearOutbox(): void {
  writeOutbox([]);
}

/**
 * Return the number of items waiting in the outbox.
 */
export function outboxSize(): number {
  return readOutbox().length;
}
