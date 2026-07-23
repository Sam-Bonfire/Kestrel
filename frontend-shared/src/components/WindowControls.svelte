<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X } from 'lucide-svelte';

  const appWindow = getCurrentWindow();

  function minimize() {
    appWindow.minimize();
  }
  
  function toggleMaximize() {
    appWindow.toggleMaximize();
  }
  
  function close() {
    appWindow.close();
  }
</script>

<div class="flex items-center gap-1 h-full shrink-0 z-[100] ml-2">
  <button onclick={minimize} class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" title="Minimize">
    <Minus class="w-3.5 h-3.5" />
  </button>
  <button onclick={toggleMaximize} class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" title="Maximize">
    <Square class="w-3.5 h-3.5" />
  </button>
  <button onclick={close} class="p-1.5 rounded-lg hover:bg-red-500/80 hover:text-white text-[var(--color-text-secondary)] transition-colors cursor-pointer" title="Close">
    <X class="w-4 h-4" />
  </button>
</div>
