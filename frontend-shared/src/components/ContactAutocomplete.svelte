<script lang="ts">
  import { X } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';

  let {
    recipients = $bindable([] as string[]),
    placeholder = 'Add recipient...'
  } = $props<{
    recipients?: string[];
    placeholder?: string;
  }>();

  interface Contact {
    id: string;
    account_id: string;
    name: string | null;
    email: string;
    avatar_url: string | null;
    last_contacted_at: number;
    created_at: number;
  }

  let inputValue = $state('');
  let searchResults = $state<Contact[]>([]);
  let isDropdownOpen = $state(false);
  let focusedIndex = $state(-1);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let inputElement: HTMLInputElement;

  $effect(() => {
    if (inputValue.trim().length > 0) {
      if (searchTimeout) clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        fetchContacts(inputValue.trim());
      }, 300);
    } else {
      searchResults = [];
      isDropdownOpen = false;
      focusedIndex = -1;
    }
  });

  async function fetchContacts(query: string) {
    try {
      // Assuming invoke or standard fetch works. Since we are in Tauri frontend-shared, typically we might fetch via API or Tauri.
      // We will use standard web fetch to the API endpoint which is running on port 1420/1421 backend depending on dev setup, or absolute paths if configured.
      // Wait, standard fetch is usually relative `/api/...`.
      const res = await fetch(`/api/contacts/search?q=${encodeURIComponent(query)}&limit=10`);
      if (res.ok) {
        let results: Contact[] = await res.json();
        // filter out existing
        results = results.filter(c => !recipients.includes(c.email));
        searchResults = results;
        isDropdownOpen = results.length > 0;
        focusedIndex = results.length > 0 ? 0 : -1;
      } else {
        searchResults = [];
        isDropdownOpen = false;
      }
    } catch (err) {
      console.error('Failed to search contacts', err);
      searchResults = [];
      isDropdownOpen = false;
    }
  }

  function addEmail(emailStr: string) {
    const emails = emailStr.split(/[\s,]+/).map(s => s.trim()).filter(Boolean);
    for (const email of emails) {
      if (email && !recipients.includes(email)) {
        recipients = [...recipients, email];
      }
    }
    inputValue = '';
    searchResults = [];
    isDropdownOpen = false;
    inputElement?.focus();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      if (isDropdownOpen && searchResults.length > 0) {
        e.preventDefault();
        focusedIndex = (focusedIndex + 1) % searchResults.length;
      }
    } else if (e.key === 'ArrowUp') {
      if (isDropdownOpen && searchResults.length > 0) {
        e.preventDefault();
        focusedIndex = focusedIndex <= 0 ? searchResults.length - 1 : focusedIndex - 1;
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (isDropdownOpen && focusedIndex >= 0 && searchResults[focusedIndex]) {
        addEmail(searchResults[focusedIndex].email);
      } else if (inputValue.trim()) {
        addEmail(inputValue);
      }
    } else if (e.key === 'Escape') {
      isDropdownOpen = false;
      focusedIndex = -1;
    } else if (e.key === ',' || e.key === 'Tab' || e.key === ' ') {
      if (inputValue.trim() && !isDropdownOpen) {
        e.preventDefault();
        addEmail(inputValue);
      } else if (isDropdownOpen && focusedIndex >= 0) {
        e.preventDefault();
        addEmail(searchResults[focusedIndex].email);
      } else if (e.key === 'Tab' && inputValue.trim()) {
         e.preventDefault();
         addEmail(inputValue);
      }
    } else if (e.key === 'Backspace' && !inputValue && recipients.length > 0) {
      recipients = recipients.slice(0, -1);
    }
  }

  function handleBlur(e: FocusEvent) {
    // delay hiding to allow clicks on dropdown
    setTimeout(() => {
      if (inputValue.trim() && !isDropdownOpen) {
        // Option: we might not want to auto-add on blur if they are just typing a name,
        // but for emails we do. We'll leave the original EmailPillInput behavior.
        addEmail(inputValue);
      }
      isDropdownOpen = false;
    }, 200);
  }

  function removeEmail(index: number) {
    recipients = recipients.filter((_: string, i: number) => i !== index);
  }
</script>

<div class="relative w-full">
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
      bind:this={inputElement}
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeyDown}
      onblur={handleBlur}
      {placeholder}
      class="flex-1 min-w-[140px] bg-transparent text-white text-xs font-sans outline-none border-none p-0 focus:ring-0 placeholder:text-[var(--color-text-secondary)]/50"
    />
  </div>

  {#if isDropdownOpen && searchResults.length > 0}
    <ul class="absolute top-full left-0 mt-1 w-full max-w-md bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-md shadow-lg overflow-hidden z-50">
      {#each searchResults as contact, i}
        <li
          class="px-3 py-2 cursor-pointer flex items-center gap-3 {i === focusedIndex ? 'bg-[var(--color-bg-hover)]' : 'hover:bg-[var(--color-bg-hover)]'}"
          onmousedown={() => addEmail(contact.email)}
        >
          <div class="w-6 h-6 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold overflow-hidden shrink-0">
            {#if contact.avatar_url}
              <img src={contact.avatar_url} alt="" class="w-full h-full object-cover" />
            {:else if contact.name}
              {contact.name.charAt(0).toUpperCase()}
            {:else}
              {contact.email.charAt(0).toUpperCase()}
            {/if}
          </div>
          <div class="flex flex-col overflow-hidden">
            {#if contact.name}
              <span class="text-sm text-white truncate">{contact.name}</span>
              <span class="text-xs text-[var(--color-text-secondary)] truncate">{contact.email}</span>
            {:else}
              <span class="text-sm text-white truncate">{contact.email}</span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
