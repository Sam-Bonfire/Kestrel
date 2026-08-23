<script lang="ts">
  import { revokedAccounts } from '../stores/auth.svelte.js';
  import { loginWithProvider } from '../api/index.js';
</script>

{#if revokedAccounts.length > 0}
  <div class="flex flex-col w-full z-50 fixed top-0 left-0">
    {#each revokedAccounts as account}
      <div class="flex items-center justify-between px-4 py-3 bg-red-500/10 border-b border-red-500/30 text-red-500 text-sm w-full">
        <span class="font-medium">
          Account sync failed for {account.provider}. The authorization may have been revoked or expired.
        </span>
        <button
          onclick={() => loginWithProvider(account.provider)}
          class="px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 border border-red-500/50 rounded-md text-red-400 font-semibold transition-colors cursor-pointer"
        >
          Re-authorize
        </button>
      </div>
    {/each}
  </div>
{/if}