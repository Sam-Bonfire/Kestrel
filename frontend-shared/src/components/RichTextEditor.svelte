<script lang="ts">
  import { Bold, Italic, Link, List } from 'lucide-svelte';
  import DOMPurify from 'dompurify';

  const SAFE_HTML = {
    ALLOWED_TAGS: ['b', 'i', 'a', 'ul', 'li', 'p', 'br', 'strong', 'em'],
    ALLOWED_ATTR: ['href', 'target'],
  } as const;

  let {
    value = $bindable(''),
    placeholder = '',
  }: {
    value?: string;
    placeholder?: string;
  } = $props();

  let editorDiv: HTMLDivElement | undefined = $state(undefined);

  function execCommand(command: string, arg?: string) {
    editorDiv?.focus();
    document.execCommand(command, false, arg);
    updateValue();
  }

  function insertLink() {
    const url = prompt('Enter link URL (https://…):');
    if (url && /^https?:\/\//i.test(url.trim())) execCommand('createLink', url.trim());
  }

  function updateValue() {
    if (editorDiv) value = editorDiv.innerHTML;
  }

  $effect(() => {
    // Reflect external resets (e.g. switching events) without clobbering typing.
    // Stored HTML is untrusted (synced from providers): sanitize before render.
    if (editorDiv && document.activeElement !== editorDiv && editorDiv.innerHTML !== value) {
      editorDiv.innerHTML = DOMPurify.sanitize(value, { ...SAFE_HTML });
    }
  });
</script>

<div class="border border-white/10 rounded-lg overflow-hidden bg-[var(--color-canvas-base)] focus-within:border-white/20 transition-all">
  <div class="flex items-center gap-1 p-1.5 bg-white/5 border-b border-white/5" role="toolbar" aria-label="Formatting">
    <button type="button" onclick={() => execCommand('bold')} title="Bold" aria-label="Bold" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors cursor-pointer">
      <Bold size={16} />
    </button>
    <button type="button" onclick={() => execCommand('italic')} title="Italic" aria-label="Italic" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors cursor-pointer">
      <Italic size={16} />
    </button>
    <div class="w-px h-4 bg-white/10 mx-1"></div>
    <button type="button" onclick={insertLink} title="Link" aria-label="Insert link" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors cursor-pointer">
      <Link size={16} />
    </button>
    <button type="button" onclick={() => execCommand('insertUnorderedList')} title="Bullet List" aria-label="Bullet list" class="p-1 hover:bg-white/10 rounded text-white/70 hover:text-white transition-colors cursor-pointer">
      <List size={16} />
    </button>
  </div>

  <div
    bind:this={editorDiv}
    contenteditable="true"
    role="textbox"
    aria-multiline="true"
    aria-label={placeholder || 'Rich text'}
    data-placeholder={placeholder}
    class="p-3 min-h-[100px] outline-none text-sm font-sans text-white/90 empty:before:content-[attr(data-placeholder)] empty:before:text-neutral-600"
    oninput={updateValue}
    onblur={updateValue}
  ></div>
</div>

<style>
  [contenteditable] a {
    color: var(--color-accent-outlook);
    text-decoration: underline;
  }
  [contenteditable] ul {
    list-style-type: disc;
    padding-left: 1.5rem;
  }
</style>
