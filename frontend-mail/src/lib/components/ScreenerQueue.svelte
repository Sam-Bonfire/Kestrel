<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { UserCheck, Ban, MailOpen } from 'lucide-svelte';
  import type { ScreenedSender } from '@kestrel/shared';
  import { isTyping } from '$lib/utils/keyboard';

  let {
    senders = [],
    onAllow = (_email: string) => {},
    onBlock = (_email: string) => {},
    onOpen = (_messageId: string) => {},
    active = false,
  } = $props<{
    senders?: ScreenedSender[];
    onAllow?: (email: string) => void;
    onBlock?: (email: string) => void;
    onOpen?: (messageId: string) => void;
    active?: boolean;
  }>();

  let sel = $state(0);

  $effect(() => {
    if (sel > senders.length - 1) sel = Math.max(0, senders.length - 1);
  });

  function onKey(e: KeyboardEvent) {
    if (!active || senders.length === 0 || isTyping(e)) return;
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const cur = senders[Math.min(sel, senders.length - 1)];
    if (!cur) return;
    if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      sel = Math.min(sel + 1, senders.length - 1);
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      sel = Math.max(sel - 1, 0);
    } else if (e.key === 'y' || e.key === 'Y') {
      onAllow(cur.email);
    } else if (e.key === 'n' || e.key === 'N') {
      onBlock(cur.email);
    } else if (e.key === 'Enter' || e.key === 'o') {
      onOpen(cur.messageId);
    }
  }

  onMount(() => {
    window.addEventListener('keydown', onKey);
  });
  onDestroy(() => {
    window.removeEventListener('keydown', onKey);
  });
</script>

<div class="flex-1 overflow-y-auto px-4 py-3 space-y-2">
  <div class="px-2 py-1 text-[10px] font-mono tracking-widest text-[var(--color-text-secondary)]/60 uppercase">
    First-time senders ({senders.length})
  </div>
  <div class="px-2 pb-1 text-[11px] text-[var(--color-text-secondary)]/70">
    Senders with a single message in recent mail. Hidden from inbox until reviewed.
    <span class="font-mono">Y</span> allow · <span class="font-mono">N</span> block · <span class="font-mono">J/K</span> move
  </div>
  {#if senders.length === 0}
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <UserCheck class="w-8 h-8 text-emerald-400/60 mb-3" />
      <span class="text-sm font-semibold text-white">No new senders</span>
      <span class="text-[11px] opacity-60">Everyone writing to you has been seen before.</span>
    </div>
  {:else}
    {#each senders as sender, idx (sender.email)}
      <div class="flex items-center gap-3 p-3 rounded-lg bg-[var(--color-canvas-base)] border transition-colors {idx === sel && active ? 'border-blue-500/60' : 'border-white/5 hover:border-white/10'}">
        <button
          type="button"
          onclick={() => onOpen(sender.messageId)}
          class="flex-1 min-w-0 text-left cursor-pointer"
          title="Open message"
        >
          <div class="text-sm font-semibold text-white truncate">{sender.name}</div>
          <div class="text-[11px] font-mono text-[var(--color-text-secondary)] truncate">&lt;{sender.email}&gt;</div>
          <div class="text-xs text-neutral-400 truncate mt-0.5">{sender.subject}</div>
        </button>
        <span class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => onAllow(sender.email)}
            title="Allow sender"
            class="p-1.5 rounded-full hover:bg-emerald-500/20 text-neutral-400 hover:text-emerald-400 transition-colors cursor-pointer"
          >
            <UserCheck class="w-4 h-4" />
          </button>
          <button
            type="button"
            onclick={() => onBlock(sender.email)}
            title="Block sender"
            class="p-1.5 rounded-full hover:bg-red-500/20 text-neutral-400 hover:text-red-400 transition-colors cursor-pointer"
          >
            <Ban class="w-4 h-4" />
          </button>
          <button
            type="button"
            onclick={() => onOpen(sender.messageId)}
            title="Open message"
            class="p-1.5 rounded-full hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer"
          >
            <MailOpen class="w-4 h-4" />
          </button>
        </span>
      </div>
    {/each}
  {/if}
</div>
