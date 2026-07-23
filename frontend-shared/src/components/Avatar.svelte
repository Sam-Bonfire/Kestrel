<script>
  export let name = '';
  export let src = '';
  export let size = 32;

  $: initials = getInitials(name);
  $: numericSize = typeof size === 'number' ? size : size === 'sm' ? 24 : size === 'lg' ? 40 : 32;
  $: fontSize = Math.round(numericSize * 0.38);

  function getInitials(fullName) {
    if (!fullName) return '?';
    const parts = fullName.trim().split(/\s+/);
    if (parts.length === 0 || !parts[0]) return '?';
    if (parts.length === 1) return parts[0].charAt(0).toUpperCase();
    return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase();
  }
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
  .avatar--fallback {
    background: rgba(255, 255, 255, 0.1);
    color: #a0a0a0;
    font-weight: 600;
    font-family: "Geist", ui-sans-serif, system-ui, sans-serif;
  }
</style>
