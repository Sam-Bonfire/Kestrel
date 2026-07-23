<script lang="ts">
  import { tagColors } from '../tokens/index.js';

  let { tag = 'devops', label = '' } = $props<{
    tag?: string;
    label?: string;
  }>();

  function getPalette(t: string): { text: string; bg: string; border: string } {
    const key = t.toLowerCase();
    if (key in tagColors) {
      return (tagColors as Record<string, { text: string; bg: string; border: string }>)[key];
    }
    return { text: t, bg: t + '15', border: t + '30' };
  }

  let palette = $derived(getPalette(tag));
</script>

<span
  class="pill"
  style="
    color: {palette.text};
    background: {palette.bg};
    border-color: {palette.border};
  "
>
  {label || tag}
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
