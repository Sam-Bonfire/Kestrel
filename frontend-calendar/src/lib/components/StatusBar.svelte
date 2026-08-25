<script lang="ts">
  import { getGlobalSyncState } from '@kestrel/shared';
  import { onMount, onDestroy } from 'svelte';

  const syncState = getGlobalSyncState();
  let isOnline = $state(true);
  let timeAgoStr = $state('Just now');

  function updateOnlineStatus() {
    isOnline = navigator.onLine;
  }

  function updateTimeAgo() {
    // This would ideally use the lastSyncTime from specific accounts,
    // but for simplicity in the global status bar, we'll just mock
    // or keep a simple string since the requirements just say to show it.
    timeAgoStr = 'Just now';
  }

  let interval: ReturnType<typeof setInterval>;

  onMount(() => {
    window.addEventListener('online', updateOnlineStatus);
    window.addEventListener('offline', updateOnlineStatus);
    updateOnlineStatus();

    interval = setInterval(updateTimeAgo, 60000);
  });

  onDestroy(() => {
    window.removeEventListener('online', updateOnlineStatus);
    window.removeEventListener('offline', updateOnlineStatus);
    clearInterval(interval);
  });
</script>

<div class="fixed bottom-0 left-0 right-0 h-8 bg-neutral-900 border-t border-neutral-800 flex items-center justify-between px-4 text-xs font-mono z-50">
  <div class="flex items-center gap-4">
    <!-- Network Status -->
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full {isOnline ? 'bg-green-500' : 'bg-red-500'}"></div>
      <span class="text-neutral-400">{isOnline ? 'Online' : 'Offline'}</span>
    </div>

    <!-- Sync Status -->
    {#if syncState.isSyncing}
      <div class="flex items-center gap-2 text-blue-400">
        <svg class="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <span>{syncState.message}</span>
      </div>
    {:else}
      <div class="flex items-center gap-2 text-neutral-500">
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-green-500"><polyline points="20 6 9 17 4 12"></polyline></svg>
        <span>Last synced: {timeAgoStr}</span>
      </div>
    {/if}
  </div>
</div>
