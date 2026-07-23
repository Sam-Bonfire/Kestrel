<script>
  import { colors } from '../tokens/index.js';

  export let provider = 'gmail';
  export let size = 'sm';

  $: config = getConfig(provider);

  function getConfig(id) {
    switch (id.toLowerCase()) {
      case 'gmail':
        return { label: 'Gmail', color: colors.accentGmail, icon: 'G' };
      case 'outlook':
        return { label: 'Outlook', color: colors.accentOutlook, icon: 'O' };
      default:
        return {
          label: id,
          color: colors.textSecondary,
          icon: id.charAt(0).toUpperCase(),
        };
    }
  }

  $: iconSize = size === 'sm' ? 18 : 24;
  $: fontSize = size === 'sm' ? 10 : 12;
</script>

<span
  class="badge badge--{size}"
  style="
    background: {config.color}15;
    border: 1px solid {config.color}30;
    color: {config.color};
  "
>
  <span
    class="badge__icon"
    style="width: {iconSize}px; height: {iconSize}px; font-size: {fontSize}px;"
  >
    {config.icon}
  </span>
  <span class="badge__label">{config.label}</span>
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    border-radius: 9999px;
    font-family: "Geist", ui-sans-serif, system-ui, sans-serif;
    font-weight: 600;
    white-space: nowrap;
    user-select: none;
  }
  .badge--sm {
    padding: 0.125rem 0.5rem;
    font-size: 0.65rem;
  }
  .badge--md {
    padding: 0.25rem 0.625rem;
    font-size: 0.75rem;
  }
  .badge__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-weight: 700;
  }
</style>
