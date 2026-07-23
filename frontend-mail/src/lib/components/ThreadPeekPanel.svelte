<script lang="ts">
  import { X, Reply, Archive, Trash2, ShieldAlert } from 'lucide-svelte';

  export let thread: {
    id: string;
    sender: string;
    subject: string;
    date: string;
    htmlBody: string;
  } | null = null;

  export let onClose: () => void = () => {};
</script>

{#if thread}
  <div class="flex-1 h-screen bg-[var(--color-canvas-card)] flex flex-col font-sans border-l border-[var(--color-border-hairline)] transition-all">
    <!-- Header Controls -->
    <div class="p-4 border-b border-[var(--color-border-hairline)] flex items-center justify-between">
      <div class="flex items-center gap-2">
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
          <Reply class="w-4 h-4" />
        </button>
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
          <Archive class="w-4 h-4" />
        </button>
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-red-400 transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>
      </div>

      <button onclick={onClose} class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Subject & Meta Header -->
    <div class="p-6 border-b border-[var(--color-border-hairline)]">
      <h1 class="text-lg font-bold text-white mb-2">{thread.subject}</h1>
      <div class="flex items-center justify-between text-xs text-[var(--color-text-secondary)]">
        <span class="font-semibold text-[var(--color-text-primary)]">{thread.sender}</span>
        <span class="font-mono">{thread.date}</span>
      </div>
    </div>

    <!-- Sandboxed HTML iframe content -->
    <div class="flex-1 p-6 bg-white overflow-hidden">
      <iframe
        title="Email content"
        sandbox="allow-same-origin"
        srcdoc={thread.htmlBody}
        class="w-full h-full border-none"
      ></iframe>
    </div>
  </div>
{:else}
  <div class="flex-1 h-screen bg-[var(--color-canvas-card)] flex items-center justify-center text-xs text-[var(--color-text-secondary)] font-mono border-l border-[var(--color-border-hairline)]">
    Select a thread to view message details
  </div>
{/if}
