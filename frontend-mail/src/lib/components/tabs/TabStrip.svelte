<script lang="ts">
  import { tabsStore } from '../../stores/tabsStore.svelte.js';
  import { Plus, X, Search, Inbox, Calendar, MessageSquare } from 'lucide-svelte';

  function getIconForView(view: string) {
    if (view === 'inbox' || view === 'all-mail') return Inbox;
    if (view.startsWith('search')) return Search;
    if (view.startsWith('calendar')) return Calendar;
    return MessageSquare; // generic
  }

  function getLabelForView(view: string, threadId: string | null | undefined) {
    if (threadId) return 'Thread ' + threadId.slice(0, 5); // Example
    if (view === 'inbox') return 'Inbox';
    if (view === 'all-mail') return 'All Mail';
    if (view.startsWith('search')) return 'Search';
    return view.charAt(0).toUpperCase() + view.slice(1);
  }
</script>

{#if tabsStore.loaded}
<div class="flex items-center bg-[#1a1919] border-b border-white/10 px-2 pt-2 h-10 w-full overflow-x-auto custom-scrollbar" role="tablist">
  {#each tabsStore.tabs as tab (tab.id)}
    {@const Icon = getIconForView(tab.view)}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      role="tab"
      aria-selected={tabsStore.activeTabId === tab.id}
      class="group flex items-center gap-2 max-w-[200px] min-w-[120px] px-3 py-1.5 rounded-t-lg border-t border-x transition-colors cursor-pointer select-none
        {tabsStore.activeTabId === tab.id
          ? 'bg-[var(--color-canvas-base)] border-white/10 border-b-transparent text-white z-10 -mb-px'
          : 'bg-[#1c1b1b] border-transparent hover:bg-white/5 text-[var(--color-text-secondary)] border-b-white/10'}"
      onclick={() => tabsStore.switchTab(tab.id)}
      onmousedown={(e: MouseEvent) => {
        if (e.button === 1) tabsStore.closeTab(tab.id); // Middle click close
      }}
    >
      <Icon class="w-3.5 h-3.5 shrink-0 opacity-70 group-hover:opacity-100 transition-opacity" />
      <span class="truncate text-xs flex-1 text-left select-none">{getLabelForView(tab.view, tab.threadId)}</span>
      <button
        class="p-0.5 rounded-sm hover:bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer border-none bg-transparent text-[var(--color-text-secondary)] hover:text-white"
        onclick={(e: MouseEvent) => { e.stopPropagation(); tabsStore.closeTab(tab.id); }}
        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') tabsStore.closeTab(tab.id); }}
        aria-label="Close tab"
      >
        <X class="w-3 h-3" />
      </button>
    </div>
  {/each}

  <button
    class="p-1.5 ml-1 rounded hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer border-none bg-transparent"
    onclick={() => tabsStore.openTab({ view: 'inbox' })}
    aria-label="New tab"
  >
    <Plus class="w-4 h-4" />
  </button>
</div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    height: 2px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }
</style>
