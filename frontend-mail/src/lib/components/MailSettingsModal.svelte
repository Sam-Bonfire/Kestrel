<script lang="ts">
  import { Settings, X } from 'lucide-svelte';
  import RichTextSignature from './RichTextSignature.svelte';
  import {
    mailDenseMode,
    mailDefaultLandingView,
    mailSignature
  } from '@kestrel/shared';

  let { isOpen = false, onClose = () => {} } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
  }>();
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs font-sans">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 cursor-pointer" onclick={onClose} />
    
    <div class="relative w-full max-w-md bg-[#131313] border border-neutral-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden z-50 text-xs text-[var(--color-text-primary)]">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-neutral-800/60 flex items-center justify-between bg-[#181818]">
        <div class="flex items-center gap-2">
          <Settings class="w-4 h-4 text-blue-400" />
          <h3 class="font-bold text-white uppercase tracking-wider">Mail Settings</h3>
        </div>
        <button onclick={onClose} class="p-1 rounded hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Scrollable Options -->
      <div class="p-6 space-y-4">
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

        <div class="space-y-1">
          <span class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Email Signature</span>
          <RichTextSignature bind:value={$mailSignature} />
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-4 border-t border-neutral-800/60 bg-[#181818] flex justify-end">
        <button onclick={onClose} class="px-4 py-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 text-white font-semibold text-xs cursor-pointer transition-colors">Done</button>
      </div>
    </div>
  </div>
{/if}
