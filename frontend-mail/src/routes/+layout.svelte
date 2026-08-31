<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { initSyncEvents, closeSyncEvents, ShortcutCheatSheet } from '@kestrel/shared';

  let { children } = $props();

  onMount(() => {
    initSyncEvents();
    return () => {
      closeSyncEvents();
    };
  });
</script>

<div class="h-screen w-screen overflow-hidden">
  {@render children()}
</div>
<ShortcutCheatSheet />
