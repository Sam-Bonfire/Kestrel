import { describe, expect, it } from 'vitest';
import { DEFAULT_CAPS, attachmentCacheKey, evictPlan } from './attachments.js';

const MINUTE = 60 * 1000;
const DAY = 24 * 60 * 60 * 1000;

describe('evictPlan', () => {
  it('expires nothing while everything is fresh and under cap', () => {
    const plan = evictPlan(
      [
        { key: 'a', size: 1000, cached_at: 0, last_access: 0 },
        { key: 'b', size: 2000, cached_at: 0, last_access: MINUTE },
      ],
      DEFAULT_CAPS,
      { key: 'c', size: 500 },
      5 * MINUTE,
    );
    expect(plan.expired).toEqual([]);
    expect(plan.evicted).toEqual([]);
  });

  it('expires entries past the age cap', () => {
    const now = 40 * DAY;
    const plan = evictPlan(
      [
        { key: 'fresh', size: 100, cached_at: 39 * DAY, last_access: 39 * DAY },
        { key: 'stale', size: 100, cached_at: 5 * DAY, last_access: 40 * DAY },
      ],
      DEFAULT_CAPS,
      { key: 'new', size: 50 },
      now,
    );
    expect(plan.expired).toEqual(['stale']);
    // The expired key is handled separately, never through the LRU pass.
    expect(plan.evicted).not.toContain('stale');
  });

  it('evicts least recently accessed first until under the size cap', () => {
    const caps = { maxTotalBytes: 5000, maxAgeMs: 30 * DAY };
    const plan = evictPlan(
      [
        { key: 'oldest-touch', size: 2000, cached_at: 0, last_access: 1000 },
        { key: 'recent-touch', size: 2000, cached_at: 0, last_access: 90000 },
        { key: 'even-older', size: 1000, cached_at: 0, last_access: 500 },
      ],
      caps,
      { key: 'new', size: 1200 },
      120000,
    );
    // Total = 5100+1200 incoming... 2000+2000+1000=5000 + 1200 = 6200 > 5000.
    // 'even-older' (500) evicted first -> 5200 still > 5000, then 'oldest-touch' -> 3200.
    expect(plan.evicted).toEqual(['even-older', 'oldest-touch']);
    expect(plan.evicted).not.toContain('recent-touch');
  });

  it('keeps the incoming entry even when it is itself the oldest touch', () => {
    const caps = { maxTotalBytes: 1000, maxAgeMs: 30 * DAY };
    const plan = evictPlan(
      [{ key: 'a', size: 800, cached_at: 0, last_access: 100 }],
      caps,
      { key: 'incoming', size: 400 },
      200,
    );
    // 1200 -> evict 'a' (only LRU candidate) instead of dropping the incoming file.
    expect(plan.evicted).toEqual(['a']);
  });

  it('sizes the incoming key out of eviction', () => {
    const caps = { maxTotalBytes: 700, maxAgeMs: 30 * DAY };
    const plan = evictPlan(
      [{ key: 'incoming', size: 400, cached_at: 1, last_access: 1 }],
      caps,
      { key: 'incoming', size: 400 },
      200,
    );
    // 800 > 700: the stored copy counts as space, so the plan targets the
    // replacement itself; cacheAttachment filters the key out of the delete
    // list and simply overwrites it.
    expect(plan.evicted).toEqual(['incoming']);
  });

  it('builds message/filename cache keys', () => {
    expect(attachmentCacheKey('m-1', 'report.pdf')).toBe('m-1/report.pdf');
  });
});
