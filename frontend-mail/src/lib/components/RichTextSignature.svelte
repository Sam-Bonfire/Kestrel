<script lang="ts">
  import { Bold, Italic, Link, List } from 'lucide-svelte';

  export let value: string = '';
  
  let editorDiv: HTMLDivElement;

  function execCommand(command: string, arg?: string) {
    document.execCommand(command, false, arg);
    updateValue();
  }

  function insertLink() {
    const url = prompt('Enter link URL:');
    if (url) {
      execCommand('createLink', url);
    }
  }

  function updateValue() {
    if (editorDiv) {
      value = editorDiv.innerHTML;
    }
  }

  // Set initial value only once when mounted
  import { onMount } from 'svelte';
  onMount(() => {
    if (editorDiv && value) {
      editorDiv.innerHTML = value;
    }
  });
</script>

<div class="rich-text-container border border-white/10 rounded-lg overflow-hidden bg-[var(--color-canvas-base)] focus-within:border-white/20 transition-all">
  <div class="toolbar flex items-center gap-1 p-2 bg-white/5 border-b border-white/5">
    <button type="button" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors" on:click={() => execCommand('bold')} title="Bold">
      <Bold size={16} />
    </button>
    <button type="button" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors" on:click={() => execCommand('italic')} title="Italic">
      <Italic size={16} />
    </button>
    <div class="w-px h-4 bg-white/10 mx-1"></div>
    <button type="button" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors" on:click={insertLink} title="Link">
      <Link size={16} />
    </button>
    <button type="button" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors" on:click={() => execCommand('insertUnorderedList')} title="Bullet List">
      <List size={16} />
    </button>
  </div>
  
  <div 
    bind:this={editorDiv}
    contenteditable="true" 
    class="editor p-3 min-h-[100px] outline-none text-sm font-sans text-white/90"
    on:input={updateValue}
    on:blur={updateValue}
  ></div>
</div>

<style>
  .editor :global(a) {
    color: var(--color-primary-highlight);
    text-decoration: underline;
  }
  .editor :global(ul) {
    list-style-type: disc;
    padding-left: 1.5rem;
  }
  .editor :global(b), .editor :global(strong) {
    font-weight: 600;
  }
</style>
