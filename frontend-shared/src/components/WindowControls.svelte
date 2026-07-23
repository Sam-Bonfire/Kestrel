<script lang="ts">
  import { onMount } from 'svelte';
  import { Minus, Square, X } from 'lucide-svelte';

  let isTauri = $state(false);

  onMount(() => {
    isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
  });

  async function minimize(e: MouseEvent) {
    e.stopPropagation();
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch (err) {
      alert(`Tauri Window Error: ${err instanceof Error ? err.message : err}. Please restart 'pnpm tauri dev' to apply the new window capabilities.`);
      console.warn('Minimize error:', err);
    }
  }

  async function toggleMaximize(e: MouseEvent) {
    e.stopPropagation();
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch (err) {
      alert(`Tauri Window Error: ${err instanceof Error ? err.message : err}. Please restart 'pnpm tauri dev' to apply the new window capabilities.`);
      console.warn('Maximize error:', err);
    }
  }

  async function closeWindow(e: MouseEvent) {
    e.stopPropagation();
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch (err) {
      alert(`Tauri Window Error: ${err instanceof Error ? err.message : err}. Please restart 'pnpm tauri dev' to apply the new window capabilities.`);
      console.warn('Close error:', err);
    }
  }
</script>

{#if isTauri}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed top-0 right-0 h-8 flex items-center z-[99999] select-none"
    onpointerdown={(e) => e.stopPropagation()}
    onmousedown={(e) => e.stopPropagation()}
    onclick={(e) => e.stopPropagation()}
  >
    <button 
      type="button"
      onclick={minimize} 
      class="w-11 h-full flex items-center justify-center hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer border-none bg-transparent active:bg-white/20" 
      title="Minimize"
    >
      <Minus class="w-3.5 h-3.5" />
    </button>
    <button 
      type="button"
      onclick={toggleMaximize} 
      class="w-11 h-full flex items-center justify-center hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer border-none bg-transparent active:bg-white/20" 
      title="Maximize"
    >
      <Square class="w-3 h-3" />
    </button>
    <button 
      type="button"
      onclick={closeWindow} 
      class="w-12 h-full flex items-center justify-center hover:bg-[#e81123] text-neutral-400 hover:text-white transition-colors cursor-pointer border-none bg-transparent active:bg-[#f1707a]" 
      title="Close"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
{/if}
