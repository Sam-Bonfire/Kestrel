<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { initSyncEvents, closeSyncEvents, ShortcutCheatSheet, initTheme } from '@kestrel/shared';

  let { children } = $props();

  onMount(() => {
    const stopTheme = initTheme();
    initSyncEvents();
    return () => {
      stopTheme();
      closeSyncEvents();
    };
  });
</script>

<div class="h-screen w-screen overflow-hidden">
  {@render children()}
</div>
<ShortcutCheatSheet />
