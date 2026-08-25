<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { initSyncEvents, closeSyncEvents } from '@kestrel/shared';
  import StatusBar from '$lib/components/StatusBar.svelte';

  let { children } = $props();

  onMount(() => {
    initSyncEvents();
    return () => {
      closeSyncEvents();
    };
  });
</script>

<div class="h-screen flex flex-col">
  <div class="flex-1 overflow-hidden">
    {@render children()}
  </div>
  <StatusBar />
</div>
