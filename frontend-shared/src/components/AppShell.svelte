<script lang="ts">
  import { Login, WindowControls } from './index.js';
  import { authState } from '../stores/auth.svelte.js';
  import { focusMode, toggleFocusMode, exitFocusMode } from '../stores/focusMode.js';
  import { Minimize2 } from 'lucide-svelte';

  import type { Snippet } from 'svelte';

  let {
    isMobileSidebarOpen = $bindable(false),
    sidebar,
    children
  } = $props<{
    isMobileSidebarOpen?: boolean;
    sidebar?: Snippet;
    children?: Snippet;
  }>();

  function isTypingTarget(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    if (!el || typeof el.tagName !== 'string') return false;
    return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT' || el.isContentEditable;
  }

  function handleFocusKeys(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && (event.key === 'F' || event.key === 'f')) {
      event.preventDefault();
      toggleFocusMode();
    } else if (event.key === 'Escape' && !isTypingTarget(event.target)) {
      exitFocusMode();
    }
  }
</script>

<svelte:window onkeydown={handleFocusKeys} />

{#if !authState.isAuthenticated || !authState.isInitialized}
  {#if !authState.isInitialized}
    <div class="flex h-screen w-screen items-center justify-center bg-[var(--color-canvas-base)]">
      <div class="animate-pulse w-8 h-8 rounded-full bg-blue-500"></div>
    </div>
  {:else}
    <Login />
  {/if}
{:else}
<div class="flex h-screen w-screen overflow-hidden bg-[var(--color-canvas-base)] relative">
  <WindowControls />

  <!-- Mobile Drawer Overlay -->
  {#if isMobileSidebarOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 bg-black/60 z-40 lg:hidden backdrop-blur-sm" onclick={() => isMobileSidebarOpen = false}></div>
  {/if}

  <!-- Sidebar Container -->
  <div class="no-print fixed inset-y-0 left-0 z-50 transform transition-all duration-300 ease-in-out lg:relative lg:translate-x-0 {isMobileSidebarOpen ? 'translate-x-0' : '-translate-x-full'} shadow-2xl lg:shadow-none {$focusMode ? 'lg:max-w-0 lg:opacity-0 lg:overflow-hidden' : 'lg:max-w-[620px]'}">
    {@render sidebar?.()}
  </div>

  <!-- Main Content Area -->
  <div class="flex-1 flex flex-col min-w-0 h-full relative">
    {@render children?.()}
  </div>

  <!-- Focus mode exit pill -->
  {#if $focusMode}
    <button
      onclick={exitFocusMode}
      title="Exit focus mode (Esc)"
      aria-label="Exit focus mode"
      class="fixed top-3 right-40 z-[60] flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-neutral-800/90 hover:bg-neutral-700 text-xs text-neutral-300 hover:text-white border border-neutral-700 shadow-xl transition-all duration-300 cursor-pointer"
    >
      <Minimize2 class="w-3.5 h-3.5" />
      <span>Exit focus (Esc)</span>
    </button>
  {/if}
</div>
{/if}
