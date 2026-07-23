const QUEUE_KEY = 'kestrel:offline:queue';

export interface QueuedMutation {
  id: string;
  path: string;
  method: string;
  body: unknown;
  createdAt: string;
}

// ── Queue CRUD ──────────────────────────────────────────────────

function readQueue(): QueuedMutation[] {
  try {
    const raw = localStorage.getItem(QUEUE_KEY);
    return raw ? (JSON.parse(raw) as QueuedMutation[]) : [];
  } catch {
    return [];
  }
}

function writeQueue(queue: QueuedMutation[]): void {
  try {
    localStorage.setItem(QUEUE_KEY, JSON.stringify(queue));
  } catch {
    // Storage full — swallow. Mutations will be lost but app stays up.
  }
}

function generateId(): string {
  return `mut_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

// ── Public API ──────────────────────────────────────────────────

/**
 * Enqueue a mutation for later replay when back online.
 * Returns the id assigned to the queued mutation.
 */
export function enqueueMutation(
  path: string,
  method: string,
  body: unknown,
): string {
  const mutation: QueuedMutation = {
    id: generateId(),
    path,
    method: method.toUpperCase(),
    body,
    createdAt: new Date().toISOString(),
  };

  const queue = readQueue();
  queue.push(mutation);
  writeQueue(queue);
  return mutation.id;
}

/**
 * Return all pending mutations ordered by creation time.
 */
export function dequeuePending(): QueuedMutation[] {
  const queue = readQueue();
  // Sort oldest first so replay is deterministic
  queue.sort(
    (a, b) =>
      new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
  );
  return queue;
}

/**
 * Remove a mutation from the queue after successful replay.
 */
export function acknowledgeMutation(id: string): void {
  const queue = readQueue().filter((m) => m.id !== id);
  writeQueue(queue);
}

/**
 * Clear all queued mutations. Use after a full resync.
 */
export function clearQueue(): void {
  writeQueue([]);
}

/**
 * Return the number of mutations waiting in the queue.
 */
export function queueSize(): number {
  return readQueue().length;
}
