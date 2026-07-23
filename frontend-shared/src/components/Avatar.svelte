<script lang="ts">
  let { name = '', src = '', size = 32 } = $props<{
    name?: string;
    src?: string;
    size?: number | string;
  }>();

  let numericSize = $derived(typeof size === 'number' ? size : size === 'sm' ? 24 : size === 'lg' ? 40 : 32);
</script>

{#if src}
  <img
    class="avatar"
    {src}
    alt={name}
    width={numericSize}
    height={numericSize}
    style="border-radius: 50%; object-fit: cover;"
  />
{/if}

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    user-select: none;
  }
</style>
