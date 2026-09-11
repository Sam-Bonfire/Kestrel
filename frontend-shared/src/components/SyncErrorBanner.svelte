<script lang="ts">
  import { getSyncErrors, dismissSyncError, getSyncState } from '../stores/sync.svelte.js';
  import { triggerSync } from '../api/index.js';

  let errors = $derived(getSyncErrors());
  let pending = $state<Record<string, boolean>>({});

  async function retry(accountId: string) {
    pending[accountId] = true;
    try {
      await triggerSync(accountId);
      dismissSyncError(accountId);
    } catch (e) {
      console.error('Manual sync retry failed', e);
      getSyncState(accountId).error = e instanceof Error ? e.message : String(e);
    } finally {
      pending[accountId] = false;
    }
  }
</script>

{#if errors.length > 0}
  <div class="flex flex-col w-full" role="alert" aria-label="Synchronization errors">
    {#each errors as { accountId, error }}
      <div class="flex items-center justify-between gap-3 px-4 py-3 bg-red-500/10 border-b border-red-500/30 text-red-400 text-sm w-full">
        <span class="font-medium truncate" title="Account {accountId}: {error}">Sync failed (account {accountId}): {error}</span>
        <span class="flex items-center gap-2 shrink-0">
          <button
            type="button"
            onclick={() => retry(accountId)}
            disabled={!!pending[accountId]}
            class="px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 disabled:opacity-50 border border-red-500/50 rounded-md text-red-300 font-semibold transition-colors cursor-pointer"
          >
            {pending[accountId] ? 'Retrying…' : 'Retry'}
          </button>
          <button
            type="button"
            onclick={() => dismissSyncError(accountId)}
            aria-label="Dismiss sync error for account {accountId}"
            class="px-3 py-1.5 hover:bg-red-500/20 border border-transparent hover:border-red-500/50 rounded-md text-red-300/70 hover:text-red-300 transition-colors cursor-pointer"
          >
            Dismiss
          </button>
        </span>
      </div>
    {/each}
  </div>
{/if}
