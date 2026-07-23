<script>
  import { tagColors } from '../tokens/index.js';

  export let tag = 'devops';
  export let label = '';

  $: palette = getPalette(tag);

  function getPalette(t) {
    if (t in tagColors) {
      return tagColors[t];
    }
    return { text: t, bg: t + '15', border: t + '30' };
  }
</script>

<span
  class="pill"
  style="
    color: {palette.text};
    background: {palette.bg};
    border-color: {palette.border};
  "
>
  {#if $$slots.default}
    <slot />
  {:else if label}
    {label}
  {:else}
    {tag}
  {/if}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    font-family: "Geist", ui-sans-serif, system-ui, sans-serif;
    font-size: 0.7rem;
    font-weight: 500;
    line-height: 1.25rem;
    white-space: nowrap;
    border: 1px solid;
    border-radius: 9999px;
    user-select: none;
  }
</style>
