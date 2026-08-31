<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { inputGuard } from '../stores/shortcuts.js';
  import { fade, scale } from 'svelte/transition';

  let isOpen = $state(false);
  let searchQuery = $state('');
  let isMac = $state(false);

  // OS detection for keyboard modifiers
  onMount(() => {
    isMac = typeof navigator !== 'undefined' && /Mac|iPod|iPhone|iPad/.test(navigator.platform);

    const handleGlobalKeydown = (e: KeyboardEvent) => {
      // Toggle cheat sheet with Shift+? (Question mark)
      if (e.key === '?' && !inputGuard(e)) {
        e.preventDefault();
        isOpen = !isOpen;
        if (isOpen) {
          searchQuery = '';
        }
      }

      // Close on Escape
      if (e.key === 'Escape' && isOpen) {
        isOpen = false;
      }
    };

    window.addEventListener('keydown', handleGlobalKeydown);
    return () => {
      window.removeEventListener('keydown', handleGlobalKeydown);
    };
  });

  // Formatting key bindings based on OS
  function formatKey(key: string) {
    if (isMac) {
      return key
        .replace(/\$mod/g, '⌘')
        .replace(/Ctrl/g, '⌃')
        .replace(/Alt/g, '⌥')
        .replace(/Shift/g, '⇧');
    } else {
      return key
        .replace(/\$mod/g, 'Ctrl');
    }
  }

  type ShortcutDef = { label: string; key: string };

  const categories: Record<string, ShortcutDef[]> = {
    'Global & Navigation': [
      { label: 'Command Palette', key: '$mod+K' },
      { label: 'Search', key: '/' },
      { label: 'Go to Inbox', key: 'I' },
      { label: 'Toggle App Sidebar', key: '[' },
      { label: 'Show Shortcuts', key: '?' },
    ],
    'Mail Actions': [
      { label: 'Archive Message', key: 'E' },
      { label: 'Delete Message', key: '#' },
      { label: 'Snooze', key: 'S' },
      { label: 'Reply', key: 'R' },
      { label: 'Reply All', key: 'A' },
      { label: 'Forward', key: 'F' },
      { label: 'Toggle Unread', key: 'U' },
      { label: 'Select Message', key: 'X' },
      { label: 'Select All', key: '* A' },
    ],
    'Calendar Actions': [
      { label: 'Today', key: 'T' },
      { label: 'Day View', key: 'D' },
      { label: 'Week View', key: 'W' },
      { label: 'Month View', key: 'M' },
      { label: 'Agenda View', key: 'A' },
      { label: 'New Event', key: 'C' },
      { label: 'Next Period', key: 'J' },
      { label: 'Previous Period', key: 'K' },
    ],
    'Composer & Editor': [
      { label: 'Compose New Message', key: 'C' },
      { label: 'Send', key: '$mod+Enter' },
      { label: 'Send & Archive', key: '$mod+Shift+Enter' },
      { label: 'Add Attachment', key: '$mod+Shift+A' },
      { label: 'Bold', key: '$mod+B' },
      { label: 'Italic', key: '$mod+I' },
      { label: 'Underline', key: '$mod+U' },
    ]
  };

  const filteredCategories = $derived.by(() => {
    if (!searchQuery.trim()) return categories;

    const query = searchQuery.toLowerCase();
    const result: Record<string, ShortcutDef[]> = {};

    for (const [category, shortcuts] of Object.entries(categories)) {
      const filtered = shortcuts.filter(s =>
        s.label.toLowerCase().includes(query) ||
        formatKey(s.key).toLowerCase().includes(query)
      );

      if (filtered.length > 0) {
        result[category] = filtered;
      }
    }

    return result;
  });

</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4" transition:fade={{ duration: 150 }} onclick={() => isOpen = false}>

    <div class="bg-[var(--color-canvas-base)] border border-[var(--color-border-subtle)] rounded-xl shadow-2xl w-full max-w-3xl max-h-[85vh] flex flex-col overflow-hidden" onclick={(e) => e.stopPropagation()} transition:scale={{ duration: 150, start: 0.95 }}>

      <!-- Header / Search -->
      <div class="p-4 border-b border-[var(--color-border-subtle)] bg-[var(--color-canvas-subtle)] flex items-center gap-3">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--color-text-muted)]"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search shortcuts..."
          class="flex-1 bg-transparent border-none outline-none text-[var(--color-text-normal)] placeholder-[var(--color-text-muted)] text-lg"
        />
        <button class="text-[var(--color-text-muted)] hover:text-[var(--color-text-normal)] transition-colors p-1" onclick={() => isOpen = false} aria-label="Close">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6 bg-[var(--color-canvas-base)]">
        {#if Object.keys(filteredCategories).length === 0}
          <div class="text-center py-12 text-[var(--color-text-muted)]">
            No shortcuts found for "{searchQuery}"
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
            {#each Object.entries(filteredCategories) as [category, shortcuts]}
              <div>
                <h3 class="text-sm font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-3">{category}</h3>
                <ul class="space-y-2">
                  {#each shortcuts as shortcut}
                    <li class="flex items-center justify-between group">
                      <span class="text-[var(--color-text-normal)] text-sm">{shortcut.label}</span>
                      <div class="flex items-center gap-1">
                        {#each formatKey(shortcut.key).split('+') as k}
                          <kbd class="px-2 py-1 bg-[var(--color-canvas-subtle)] border border-[var(--color-border-subtle)] rounded text-xs font-mono text-[var(--color-text-muted)] group-hover:border-[var(--color-border-strong)] transition-colors">{k.trim()}</kbd>
                        {/each}
                      </div>
                    </li>
                  {/each}
                </ul>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    </div>
  </div>
{/if}