// Offline attachment cache (K-1567): previously downloaded attachments stay
// retrievable without network. Bytes live in IndexedDB via idb-keyval; the
// caps and eviction order are enforced by evictPlan() (pure, unit-tested).
// Every function is best-effort: a cache miss or failure must never break a
// download.
//
// shortcut: store uses one key-value bucket, no schema; if per-account
// partitioning is ever needed, upgrade keys to `${accountId}/${messageId}/${fn}`.
import { createStore, get, set, del, entries, type UseStore } from 'idb-keyval';

export type AttachmentCacheRecord = {
  key: string; // `${messageId}/${filename}`
  blob: Blob;
  size: number;
  cached_at: number; // epoch ms
  last_access: number; // epoch ms
};

export type EvictionCaps = {
  maxTotalBytes: number; // total stored bytes allowed
  maxAgeMs: number; // older entries are expired regardless of space
};

export const DEFAULT_CAPS: EvictionCaps = {
  maxTotalBytes: 50 * 1024 * 1024,
  maxAgeMs: 30 * 24 * 60 * 60 * 1000,
};

export type EvictPlan = {
  expired: string[]; // too old
  evicted: string[]; // LRU-evicted for space
};

export function evictPlan(
  entriesInput: Array<Pick<AttachmentCacheRecord, 'key' | 'size' | 'last_access' | 'cached_at'>>,
  caps: EvictionCaps,
  incoming: { key: string; size: number },
  now: number,
): EvictPlan {
  const expired = entriesInput.filter((e) => now - e.cached_at > caps.maxAgeMs).map((e) => e.key);
  const keep = entriesInput.filter((e) => !expired.includes(e.key));

  // LRU order: least recently accessed evicted first.
  const byLru = keep.slice().sort((a, b) => a.last_access - b.last_access);
  const evicted: string[] = [];
  let total = incoming.size + byLru.reduce((sum, e) => sum + e.size, 0);
  let i = 0;
  while (total > caps.maxTotalBytes && i < byLru.length) {
    total -= byLru[i].size;
    evicted.push(byLru[i].key);
    i++;
  }
  return { expired, evicted };
}

let store: UseStore | undefined;
function getStore(): UseStore {
  if (!store) store = createStore('kestrel-offline', 'attachments');
  return store;
}

export function attachmentCacheKey(messageId: string, filename: string): string {
  return `${messageId}/${filename}`;
}

export async function getCachedAttachment(
  messageId: string,
  filename: string,
): Promise<Blob | undefined> {
  try {
    const key = attachmentCacheKey(messageId, filename);
    const rec = await get<AttachmentCacheRecord>(key, getStore());
    if (!rec) return undefined;
    void set(key, { ...rec, last_access: Date.now() }, getStore()).catch(() => {});
    return rec.blob;
  } catch {
    return undefined;
  }
}

export async function cacheAttachment(
  messageId: string,
  filename: string,
  bytes: BlobPart,
  caps: EvictionCaps = DEFAULT_CAPS,
): Promise<void> {
  try {
    const blob = new Blob([bytes]);
    const key = attachmentCacheKey(messageId, filename);
    const pairs = await entries(getStore());
    const records = pairs
      .map(([, v]) => v as AttachmentCacheRecord)
      .filter((v): v is AttachmentCacheRecord => !!v && typeof v.size === 'number');
    const now = Date.now();
    const { expired, evicted } = evictPlan(records, caps, { key, size: blob.size }, now);
    const doomed = [...expired, ...evicted].filter((k) => k !== key);
    for (const doomedKey of doomed) await del(doomedKey, getStore()).catch(() => {});
    await set(key, { key, blob, size: blob.size, cached_at: now, last_access: now }, getStore());
  } catch {
    // Cache write is best-effort; the user still has their file.
  }
}

// Bytes past the age cap are purged. Cheap to call on app start.
export async function pruneAttachmentCache(caps: EvictionCaps = DEFAULT_CAPS): Promise<void> {
  try {
    const pairs = await entries(getStore());
    const now = Date.now();
    for (const [key, v] of pairs) {
      const rec = v as AttachmentCacheRecord;
      if (rec && now - rec.cached_at > caps.maxAgeMs) {
        await del(key, getStore()).catch(() => {});
      }
    }
  } catch {
    // No cache available; nothing to prune.
  }
}
