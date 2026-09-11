import { describe, it, expect } from 'vitest';
import { getSyncState, getSyncErrors, dismissSyncError } from './sync.svelte.js';

describe('Sync error tracking', () => {
  it('lists accounts with errors', () => {
    getSyncState('sync-err-a').error = 'connection refused';
    expect(getSyncErrors()).toContainEqual({ accountId: 'sync-err-a', error: 'connection refused' });
    dismissSyncError('sync-err-a');
  });

  it('omits accounts without errors', () => {
    getSyncState('sync-ok-b');
    expect(getSyncErrors().find((e) => e.accountId === 'sync-ok-b')).toBeUndefined();
  });

  it('dismiss clears the error', () => {
    getSyncState('sync-err-c').error = 'timeout';
    dismissSyncError('sync-err-c');
    expect(getSyncErrors().find((e) => e.accountId === 'sync-err-c')).toBeUndefined();
  });

  it('dismiss of unknown account is a no-op', () => {
    expect(() => dismissSyncError('sync-missing-d')).not.toThrow();
  });
});
