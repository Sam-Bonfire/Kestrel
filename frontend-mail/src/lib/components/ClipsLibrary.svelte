<script lang="ts">
  import { Trash2, MailOpen, Search } from 'lucide-svelte';
  import type { Clip } from '@kestrel/shared/api';

  let {
    clips = [],
    onOpen = (_messageId: string) => {},
    onDelete = (_id: string) => {},
  } = $props<{
    clips?: Clip[];
    onOpen?: (messageId: string) => void;
    onDelete?: (id: string) => void;
  }>();

  let query = $state('');

  let filtered = $derived(
    query.trim()
      ? clips.filter((c: Clip) => c.snippet.toLowerCase().includes(query.trim().toLowerCase()))
      : clips
  );
</script>

<div class="flex-1 overflow-y-auto px-4 py-3 space-y-2">
  <div class="px-2 py-1 text-[10px] font-mono tracking-widest text-[var(--color-text-secondary)]/60 uppercase">
    Clips ({clips.length})
  </div>
  <div class="px-2 pb-1">
    <div class="flex items-center gap-2 rounded-lg bg-white/5 border border-white/10 px-2.5 py-1.5">
      <Search class="w-3.5 h-3.5 text-neutral-500 shrink-0" />
      <input
        bind:value={query}
        placeholder="Search clips…"
        aria-label="Search clips"
        class="w-full bg-transparent text-xs text-white outline-none placeholder:text-neutral-600"
      />
    </div>
  </div>
  {#if filtered.length === 0}
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <span class="text-sm font-semibold text-white">
        {clips.length === 0 ? 'No clips yet' : 'No matches'}
      </span>
      <span class="text-[11px] opacity-60">
        {clips.length === 0
          ? 'Select text in a message and hit the scissors to clip it.'
          : 'Try a different search.'}
      </span>
    </div>
  {:else}
    {#each filtered as clip (clip.id)}
      <div class="rounded-lg bg-[var(--color-canvas-base)] border border-white/5 hover:border-white/10 transition-colors p-3">
        <p class="text-xs text-white/90 whitespace-pre-wrap line-clamp-4">{clip.snippet}</p>
        <div class="mt-2 flex items-center justify-end gap-1">
          <button
            type="button"
            onclick={() => onOpen(clip.message_id)}
            title="Jump to source message"
            class="flex items-center gap-1 px-2 py-1 text-[11px] rounded hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer"
          >
            <MailOpen class="w-3.5 h-3.5" /> Source
          </button>
          <button
            type="button"
            onclick={() => onDelete(clip.id)}
            title="Delete clip"
            class="p-1.5 rounded-full hover:bg-red-500/20 text-neutral-500 hover:text-red-400 transition-colors cursor-pointer"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
