<script lang="ts">
  import { Settings, X, Plus, Trash2 } from 'lucide-svelte';
  import RichTextSignature from './RichTextSignature.svelte';
  import {
    mailDenseMode,
    mailDefaultLandingView,
    templateStore
  } from '@kestrel/shared';
  import { apiClient } from '@kestrel/shared/api';
  import { onMount } from 'svelte';

  let { isOpen = false, onClose = () => {} } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
  }>();

  let activeTab = $state<'general' | 'snippets' | 'signatures'>('general');
  let accounts = $state<any[]>([]);

  onMount(async () => {
    try {
      accounts = await apiClient.get('/api/v1/accounts');
    } catch (e) {
      console.error('Failed to fetch accounts', e);
    }
  });

  function addSnippet() {
    templateStore.snippets = [...templateStore.snippets, {
      id: crypto.randomUUID(),
      title: 'New Snippet',
      shortcut: '/new',
      template: ''
    }];
  }

  function deleteSnippet(id: string) {
    templateStore.snippets = templateStore.snippets.filter(s => s.id !== id);
  }

  function addSignature() {
    templateStore.signatures = [...templateStore.signatures, {
      id: crypto.randomUUID(),
      accountId: accounts.length > 0 ? accounts[0].id : null,
      name: 'New Signature',
      htmlContent: '',
      isDefault: false
    }];
  }

  function deleteSignature(id: string) {
    templateStore.signatures = templateStore.signatures.filter(s => s.id !== id);
  }

  let wasOpen = $state(false);

  // Effect to sync changes whenever modal closes
  $effect(() => {
    if (isOpen) {
      wasOpen = true;
      // Load latest when opened
      templateStore.initializeTemplates();
    } else if (wasOpen && !isOpen) {
      // Sync on close
      wasOpen = false;
      templateStore.syncTemplates();
    }
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs font-sans">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 cursor-pointer" onclick={onClose} />
    
    <div class="relative w-full max-w-2xl bg-[#131313] border border-neutral-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden z-50 text-xs text-[var(--color-text-primary)]" style="max-height: 90vh;">
      <!-- Header -->
      <div class="px-5 pt-4 border-b border-neutral-800/60 bg-[#181818]">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <Settings class="w-4 h-4 text-blue-400" />
            <h3 class="font-bold text-white uppercase tracking-wider">Mail Settings</h3>
          </div>
          <button onclick={onClose} class="p-1 rounded hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="flex gap-4">
          <button onclick={() => activeTab = 'general'} class="pb-2 px-1 border-b-2 transition-colors {activeTab === 'general' ? 'border-blue-500 text-white' : 'border-transparent text-neutral-400 hover:text-white'}">General</button>
          <button onclick={() => activeTab = 'snippets'} class="pb-2 px-1 border-b-2 transition-colors {activeTab === 'snippets' ? 'border-blue-500 text-white' : 'border-transparent text-neutral-400 hover:text-white'}">Snippets</button>
          <button onclick={() => activeTab = 'signatures'} class="pb-2 px-1 border-b-2 transition-colors {activeTab === 'signatures' ? 'border-blue-500 text-white' : 'border-transparent text-neutral-400 hover:text-white'}">Signatures</button>
        </div>
      </div>

      <!-- Scrollable Options -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        {#if activeTab === 'general'}
          <label class="flex items-center justify-between p-3 bg-neutral-900/35 border border-white/5 rounded-xl cursor-pointer">
            <div class="space-y-0.5">
              <span class="font-semibold text-white">Dense Layout Mode</span>
              <p class="text-[10px] text-[var(--color-text-secondary)]">Narrower heights for list elements.</p>
            </div>
            <input type="checkbox" bind:checked={$mailDenseMode} class="accent-blue-500 rounded cursor-pointer" />
          </label>

          <div class="space-y-1">
            <span class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Default Landing View</span>
            <select bind:value={$mailDefaultLandingView} class="w-full bg-[var(--color-canvas-base)] text-white rounded-lg p-2.5 outline-none border border-white/10 focus:border-white/20 transition-all cursor-pointer">
              <option value="inbox">Inbox Folder</option>
              <option value="unread">Unread Feed</option>
              <option value="starred">Starred List</option>
              <option value="all-mail">All Mail View</option>
            </select>
          </div>
        {/if}

        {#if activeTab === 'snippets'}
          <div class="flex justify-between items-center mb-2">
            <span class="font-semibold text-white">Canned Snippets</span>
            <button onclick={addSnippet} class="flex items-center gap-1 px-2 py-1 bg-white/10 hover:bg-white/20 rounded text-white transition-colors">
              <Plus class="w-3.5 h-3.5" /> Add Snippet
            </button>
          </div>
          {#if templateStore.snippets.length === 0}
            <div class="text-center p-6 bg-white/5 rounded-xl border border-white/5 text-neutral-400">
              No snippets configured yet. Type shortcuts like /thanks to insert templates.
            </div>
          {/if}
          <div class="space-y-4">
            {#each templateStore.snippets as snippet (snippet.id)}
              <div class="p-4 bg-neutral-900/50 border border-white/10 rounded-xl space-y-3 relative">
                <button onclick={() => deleteSnippet(snippet.id)} class="absolute top-4 right-4 text-neutral-500 hover:text-red-400">
                  <Trash2 class="w-4 h-4" />
                </button>
                <div class="grid grid-cols-2 gap-3 pr-8">
                  <div>
                    <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">Title</label>
                    <input type="text" bind:value={snippet.title} class="w-full bg-[var(--color-canvas-base)] text-white rounded p-1.5 outline-none border border-white/10 focus:border-blue-500/50" />
                  </div>
                  <div>
                    <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">Shortcut</label>
                    <input type="text" bind:value={snippet.shortcut} class="w-full bg-[var(--color-canvas-base)] text-white rounded p-1.5 outline-none border border-white/10 focus:border-blue-500/50" placeholder="e.g. /thanks" />
                  </div>
                </div>
                <div>
                  <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">Template String</label>
                  <textarea bind:value={snippet.template} class="w-full h-20 bg-[var(--color-canvas-base)] text-white rounded p-2 outline-none border border-white/10 focus:border-blue-500/50 resize-none font-mono text-[11px]" placeholder="Use {{name}} for variables..."></textarea>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if activeTab === 'signatures'}
          <div class="flex justify-between items-center mb-2">
            <span class="font-semibold text-white">Email Signatures</span>
            <button onclick={addSignature} class="flex items-center gap-1 px-2 py-1 bg-white/10 hover:bg-white/20 rounded text-white transition-colors">
              <Plus class="w-3.5 h-3.5" /> Add Signature
            </button>
          </div>
          {#if templateStore.signatures.length === 0}
            <div class="text-center p-6 bg-white/5 rounded-xl border border-white/5 text-neutral-400">
              No signatures configured yet.
            </div>
          {/if}
          <div class="space-y-4">
            {#each templateStore.signatures as sig (sig.id)}
              <div class="p-4 bg-neutral-900/50 border border-white/10 rounded-xl space-y-3 relative">
                <button onclick={() => deleteSignature(sig.id)} class="absolute top-4 right-4 text-neutral-500 hover:text-red-400">
                  <Trash2 class="w-4 h-4" />
                </button>
                <div class="grid grid-cols-2 gap-3 pr-8">
                  <div>
                    <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">Name</label>
                    <input type="text" bind:value={sig.name} class="w-full bg-[var(--color-canvas-base)] text-white rounded p-1.5 outline-none border border-white/10 focus:border-blue-500/50" />
                  </div>
                  <div>
                    <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">Default For Account</label>
                    <select bind:value={sig.accountId} class="w-full bg-[var(--color-canvas-base)] text-white rounded p-1.5 outline-none border border-white/10 focus:border-blue-500/50">
                      <option value={null}>None</option>
                      {#each accounts as acc}
                        <option value={acc.id}>{acc.email_address} ({acc.provider})</option>
                      {/each}
                    </select>
                  </div>
                </div>
                <label class="flex items-center gap-2 cursor-pointer mt-1">
                  <input type="checkbox" bind:checked={sig.isDefault} class="accent-blue-500 rounded cursor-pointer" />
                  <span class="text-[11px] text-neutral-300">Set as default signature for this account</span>
                </label>
                <div class="pt-2">
                  <label class="block text-[10px] text-neutral-400 mb-1 uppercase tracking-wider">HTML Content</label>
                  <RichTextSignature bind:value={sig.htmlContent} />
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-4 border-t border-neutral-800/60 bg-[#181818] flex justify-end">
        <button onclick={onClose} class="px-4 py-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 text-white font-semibold text-xs cursor-pointer transition-colors">Done</button>
      </div>
    </div>
  </div>
{/if}
