<script lang="ts">
  import { currentCrumb, humanizeCrumb } from '../stores/breadcrumbs.js';

  let { app = '' }: { app?: string } = $props();

  let view = $derived(humanizeCrumb($currentCrumb));
</script>

<nav aria-label="Breadcrumb" class="flex items-center gap-1.5 px-6 py-2 text-xs text-neutral-500 border-b border-white/5 bg-[#131313]">
  {#if app}
    <span class="truncate">{app}</span>
    <span aria-hidden="true" class="text-neutral-700">/</span>
  {/if}
  <span aria-current="page" class="text-neutral-200 font-medium truncate">{view}</span>
</nav>
