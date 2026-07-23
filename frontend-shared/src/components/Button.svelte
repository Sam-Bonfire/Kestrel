<script lang="ts">
  import { buttonStyles } from '../tokens/index.js';

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    children,
    onclick
  } = $props<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    loading?: boolean;
    children?: import('svelte').Snippet;
    onclick?: (e: MouseEvent) => void;
  }>();

  let base = $derived(buttonStyles[variant as keyof typeof buttonStyles] || buttonStyles.primary);
  let opacity = $derived(disabled ? 'opacity: 0.5; pointer-events: none;' : '');
  let sizePad = $derived(
    size === 'sm'
      ? 'padding: 0.25rem 0.5rem; font-size: 0.7rem;'
      : `padding: ${('paddingY' in base ? base.paddingY : undefined) ?? '0.375rem'} ${('paddingX' in base ? base.paddingX : undefined) ?? '0.75rem'}; font-size: ${('fontSize' in base ? base.fontSize : undefined) ?? '0.75rem'};`
  );
  let border = $derived(
    'borderColor' in base
      ? `border: 1px solid ${(base as any).borderColor};`
      : variant === 'danger'
        ? `border: 1px solid ${buttonStyles.danger.borderColor};`
        : ''
  );
  let style = $derived([
    `background: ${('bg' in base ? base.bg : undefined) ?? 'transparent'};`,
    `color: ${base.textColor};`,
    `font-weight: ${('fontWeight' in base ? base.fontWeight : undefined) ?? '500'};`,
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
  ].join(' '));
</script>

<button {style} {disabled} {onclick}>
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
  {#if children}
    {@render children()}
  {/if}
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
