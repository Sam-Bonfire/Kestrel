<script>
  import { buttonStyles } from '../tokens/index.js';

  export let variant = 'primary';
  export let size = 'md';
  export let disabled = false;
  export let loading = false;

  $: base = buttonStyles[variant] || buttonStyles.primary;
  $: opacity = disabled ? 'opacity: 0.5; pointer-events: none;' : '';
  $: sizePad =
    size === 'sm'
      ? 'padding: 0.25rem 0.5rem; font-size: 0.7rem;'
      : `padding: ${base.paddingY ?? '0.375rem'} ${base.paddingX ?? '0.75rem'}; font-size: ${base.fontSize};`;
  $: border =
    'borderColor' in base
      ? `border: 1px solid ${base.borderColor};`
      : variant === 'danger'
        ? `border: 1px solid ${buttonStyles.danger.borderColor};`
        : '';
  $: style = [
    `background: ${base.bg ?? 'transparent'};`,
    `color: ${base.textColor};`,
    `font-weight: ${base.fontWeight ?? '500'};`,
    `border-radius: ${base.borderRadius};`,
    sizePad,
    border,
    opacity,
    'cursor: pointer;',
    'display: inline-flex;',
    'align-items: center;',
    'gap: 0.375rem;',
    'transition: background 120ms, color 120ms;',
    'white-space: nowrap;',
  ].join(' ');
</script>

<button {style} {disabled} on:click>
  {#if loading}
    <svg
      class="spinner"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2.5"
    >
      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
    </svg>
  {/if}
  <slot />
</button>

<style>
  button:hover {
    filter: brightness(0.92);
  }
  .spinner {
    animation: spinSlow 2s linear infinite;
  }
  @keyframes spinSlow {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
