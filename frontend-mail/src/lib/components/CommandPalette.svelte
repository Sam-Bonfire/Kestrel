<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, Mail, Settings, PenSquare } from 'lucide-svelte';

  let {
    isOpen = false,
    onClose = () => {},
    onSelectCommand = (cmd: string) => {}
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSelectCommand?: (cmd: string) => void;
  }>();

  let query = $state('');

  const commands = [
    { id: 'compose', title: 'Compose New Message', category: 'Actions', icon: PenSquare },
    { id: 'inbox', title: 'Go to Inbox', category: 'Navigation', icon: Mail },
    { id: 'settings', title: 'Open Settings', category: 'Navigation', icon: Settings },
  ];

  let filteredCommands = $derived(
    commands.filter(c => c.title.toLowerCase().includes(query.toLowerCase()))
  );

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      isOpen = !isOpen;
    } else if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-start justify-center pt-20 p-4 bg-black/60 backdrop-blur-xs">
    <div class="w-full max-w-xl bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl overflow-hidden font-sans">
      <!-- Search Input -->
      <div class="p-3 border-b border-[var(--color-border-hairline)] flex items-center gap-3">
        <Search class="w-4 h-4 text-[var(--color-text-secondary)]" />
        <input
          type="text"
          bind:value={query}
          placeholder="Type a command or search..."
          class="w-full bg-transparent text-white text-xs outline-none"
        />
      </div>

      <!-- Commands List -->
      <div class="max-h-64 overflow-y-auto p-2 space-y-1">
        {#each filteredCommands as cmd}
          <button
            onclick={() => { onSelectCommand(cmd.id); onClose(); }}
            class="w-full flex items-center justify-between p-2.5 rounded-lg text-xs text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)] transition-colors cursor-pointer"
          >
            <div class="flex items-center gap-2.5">
              <cmd.icon class="w-4 h-4 text-[var(--color-text-secondary)]" />
              <span>{cmd.title}</span>
            </div>
            <span class="text-[10px] font-mono text-[var(--color-text-secondary)]">{cmd.category}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}
