<script lang="ts">
  import { Login, WindowControls } from './index.js';
  import { authState } from '../stores/auth.svelte.js';

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
</script>

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
  <div class="fixed inset-y-0 left-0 z-50 transform transition-transform duration-300 lg:transform-none lg:relative lg:translate-x-0 {isMobileSidebarOpen ? 'translate-x-0' : '-translate-x-full'} shadow-2xl lg:shadow-none">
    {@render sidebar?.()}
  </div>

  <!-- Main Content Area -->
  <div class="flex-1 flex flex-col min-w-0 h-full relative">
    {@render children?.()}
  </div>
</div>
{/if}
