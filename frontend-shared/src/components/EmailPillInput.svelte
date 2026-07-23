<script lang="ts">
  import { X } from 'lucide-svelte';

  let {
    recipients = $bindable([] as string[]),
    placeholder = 'Add recipient email...'
  } = $props<{
    recipients?: string[];
    placeholder?: string;
  }>();

  let inputValue = $state('');

  function addEmail(emailStr: string) {
    const emails = emailStr.split(/[\s,]+/).map(s => s.trim()).filter(Boolean);
    for (const email of emails) {
      if (email && !recipients.includes(email)) {
        recipients = [...recipients, email];
      }
    }
    inputValue = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',' || e.key === 'Tab' || e.key === ' ') {
      if (inputValue.trim()) {
        e.preventDefault();
        addEmail(inputValue);
      }
    } else if (e.key === 'Backspace' && !inputValue && recipients.length > 0) {
      recipients = recipients.slice(0, -1);
    }
  }

  function handleBlur() {
    if (inputValue.trim()) {
      addEmail(inputValue);
    }
  }

  function removeEmail(index: number) {
    recipients = recipients.filter((_: string, i: number) => i !== index);
  }
</script>

<div class="flex flex-wrap items-center gap-1.5 min-h-[30px] w-full cursor-text py-0.5">
  {#each recipients as email, i}
    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-blue-500/15 text-blue-300 border border-blue-500/25 text-xs font-mono select-none animate-fadeIn">
      <span>{email}</span>
      <button
        type="button"
        onclick={() => removeEmail(i)}
        class="hover:bg-blue-500/30 p-0.5 rounded text-blue-400 hover:text-white transition-colors cursor-pointer"
        title="Remove"
      >
        <X class="w-3 h-3" />
      </button>
    </span>
  {/each}

  <input
    type="text"
    bind:value={inputValue}
    onkeydown={handleKeyDown}
    onblur={handleBlur}
    {placeholder}
    class="flex-1 min-w-[140px] bg-transparent text-white text-xs font-sans outline-none border-none p-0 focus:ring-0 placeholder:text-[var(--color-text-secondary)]/50"
  />
</div>
