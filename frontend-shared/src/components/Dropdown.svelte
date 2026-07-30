<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  let {
    trigger,
    content,
    isOpen = false,
    onClose
  } = $props<{
    trigger: import('svelte').Snippet;
    content: import('svelte').Snippet;
    isOpen: boolean;
    onClose: () => void;
  }>();
</script>

<div class="relative inline-flex">
  <!-- Trigger wrapper -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div onclick={(e) => e.stopPropagation()}>
    {@render trigger()}
  </div>

  {#if isOpen}
    <!-- Invisible overlay to catch clicks outside -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={onClose}></div>
    
    <!-- Dropdown Content -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="absolute right-0 bottom-full md:bottom-auto md:top-full mb-2 md:mt-2 min-w-[200px] w-56 bg-[var(--color-canvas-modal)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl z-50 flex flex-col max-h-[300px] overflow-hidden"
      transition:fly={{ y: -5, duration: 150, easing: cubicOut }}
      onclick={(e) => e.stopPropagation()}
    >
      {@render content()}
    </div>
  {/if}
</div>
