<script lang="ts">
  import { recentBreadcrumbs, MAX_CRUMBS, humanizeCrumb } from '../stores/breadcrumbs.js';

  // Chronological order with the current (newest) view last.
  let ordered = $derived([...$recentBreadcrumbs].reverse().slice(-MAX_CRUMBS));
</script>

{#if ordered.length > 1}
  <nav aria-label="Breadcrumb" class="flex items-center gap-1.5 px-6 py-2 text-xs text-neutral-500 border-b border-white/5 bg-[#131313]">
    {#each ordered as crumb, i}
      {#if i > 0}
        <span aria-hidden="true" class="text-neutral-700">/</span>
      {/if}
      {#if i === ordered.length - 1}
        <span aria-current="page" class="text-neutral-200 font-medium truncate">{humanizeCrumb(crumb.label)}</span>
      {:else}
        <span class="truncate">{humanizeCrumb(crumb.label)}</span>
      {/if}
    {/each}
  </nav>
{/if}
