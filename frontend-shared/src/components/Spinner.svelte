<script lang="ts">
  let { size = 20, strokeWidth = 2.5 } = $props<{
    size?: number | string;
    strokeWidth?: number | string;
  }>();
</script>

<svg
  class="spinner"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
>
  <path d="M12 2a10 10 0 0 1 10 10" />
</svg>

<style>
  .spinner {
    animation: spinSlow 2s linear infinite;
    color: currentColor;
  }
  @keyframes spinSlow {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
