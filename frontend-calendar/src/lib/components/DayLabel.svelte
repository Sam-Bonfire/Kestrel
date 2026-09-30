<script lang="ts">
  import { getDayLabel, setDayLabel } from '@kestrel/shared';

  let { dateStr = '' }: { dateStr?: string } = $props();

  let editing = $state(false);
  let draft = $state('');
  let label = $state<string | null>(null);

  $effect(() => {
    // Refresh when navigating to another date; never write a stale
    // draft to a newly shown day.
    label = getDayLabel(dateStr);
    editing = false;
    draft = '';
  });

  function save() {
    if (draft.trim()) label = setDayLabel(dateStr, draft);
    editing = false;
  }
</script>

{#if editing}
  <input
    bind:value={draft}
    maxlength="60"
    aria-label="Day label"
    placeholder="Label this day…"
    class="mt-1 w-full max-w-[110px] rounded bg-white/10 px-1.5 py-0.5 text-[10px] text-white outline-none focus:ring-1 focus:ring-blue-500"
    onkeydown={(e) => {
      if (e.key === 'Enter') save();
      else if (e.key === 'Escape') editing = false;
    }}
    onblur={() => {
      // Click-away keeps a non-empty draft; Escape or empty discards.
      if (draft.trim()) save();
      else editing = false;
    }}
  />
{:else if label}
  <button
    onclick={() => {
      draft = label ?? '';
      editing = true;
    }}
    title="Edit day label"
    class="mt-1 max-w-[110px] truncate text-[10px] font-mono text-amber-200/80 hover:text-amber-200 cursor-pointer"
  >
    {label}
  </button>
{:else}
  <button
    onclick={() => {
      draft = '';
      editing = true;
    }}
    title="Add day label"
    aria-label="Add day label"
    class="mt-1 text-[10px] text-neutral-600 hover:text-neutral-300 opacity-0 hover:opacity-100 focus-visible:opacity-100 transition-opacity cursor-pointer"
  >
    +
  </button>
{/if}
