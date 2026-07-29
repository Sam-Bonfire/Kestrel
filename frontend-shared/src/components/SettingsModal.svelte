<script lang="ts">
  import { X } from 'lucide-svelte';

  export let isOpen = false;
  export let onClose = () => {};

  // For Task 47: Shortcut Rebinding
  // This is a simple settings panel for Shortcuts
  let shortcuts = [
    { id: 'compose', label: 'Compose New Message', key: 'C' },
    { id: 'command', label: 'Command Palette', key: 'Cmd/Ctrl + K' },
    { id: 'day_view', label: 'Day View (Calendar)', key: 'D' },
    { id: 'week_view', label: 'Week View (Calendar)', key: 'W' },
  ];

  let recordingId: string | null = null;
  let recordingKey = '';

  function startRecording(id: string) {
    recordingId = id;
    recordingKey = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recordingId) return;
    
    e.preventDefault();
    e.stopPropagation();

    // Ignore modifiers by themselves
    if (['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) return;

    let keys = [];
    if (e.metaKey) keys.push('Cmd');
    if (e.ctrlKey) keys.push('Ctrl');
    if (e.altKey) keys.push('Alt');
    if (e.shiftKey) keys.push('Shift');
    keys.push(e.key.toUpperCase());

    const combo = keys.join(' + ');
    
    shortcuts = shortcuts.map(s => s.id === recordingId ? { ...s, key: combo } : s);
    recordingId = null;
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs" onkeydown={handleKeyDown} tabindex="-1">
    <div class="w-full max-w-2xl bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl overflow-hidden font-sans" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-border-hairline)]">
        <h2 class="text-lg font-semibold text-white">Settings</h2>
        <button onclick={onClose} class="p-1 hover:bg-white/10 rounded-md text-[var(--color-text-secondary)] transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>
      
      <div class="p-4 flex h-[400px]">
        <!-- Sidebar -->
        <div class="w-48 border-r border-[var(--color-border-hairline)] pr-4">
          <button class="w-full text-left px-3 py-2 rounded-md bg-[var(--color-canvas-hover)] text-white text-sm font-medium">
            Keyboard Shortcuts
          </button>
        </div>
        
        <!-- Content -->
        <div class="flex-1 pl-4 overflow-y-auto">
          <h3 class="text-white font-medium mb-4">Custom Shortcuts</h3>
          <p class="text-[var(--color-text-secondary)] text-sm mb-6">Click on a shortcut to record a new key combination.</p>
          
          <div class="space-y-3">
            {#each shortcuts as shortcut}
              <div class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)]">
                <span class="text-sm text-white">{shortcut.label}</span>
                <button
                  onclick={() => startRecording(shortcut.id)}
                  class="px-3 py-1.5 rounded-md text-xs font-mono font-medium border border-white/20 hover:border-white/40 transition-colors {recordingId === shortcut.id ? 'bg-blue-500/20 border-blue-500 text-blue-400' : 'bg-[#1a1919] text-[var(--color-text-secondary)]'}"
                >
                  {recordingId === shortcut.id ? 'Recording...' : shortcut.key}
                </button>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
