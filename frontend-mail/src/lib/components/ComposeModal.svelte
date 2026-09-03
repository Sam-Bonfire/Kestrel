<script lang="ts">
  import { X, Send, Paperclip, Bold, Italic, List, Link } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { ContactAutocomplete } from '@kestrel/shared';
  import { apiClient } from '@kestrel/shared/api';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { templateStore } from '@kestrel/shared';
  import type { Account } from '@kestrel/shared/api';

  let {
    isOpen = false,
    onClose = () => {},
    onSend = (draft: { to: string; cc?: string; bcc?: string; subject: string; body: string }) => {},
    initialTo = [],
    initialSubject = '',
    initialBody = '',
    initialAttachments = [] as { filename: string; content_type: string; base64_content: string; size: number }[]
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSend?: (draft: { to: string; cc?: string; bcc?: string; subject: string; body: string }) => void;
    initialTo?: string[];
    initialSubject?: string;
    initialBody?: string;
    initialAttachments?: { filename: string; content_type: string; base64_content: string; size: number }[];
  }>();

  let toRecipients = $state<string[]>(initialTo);
  let ccRecipients = $state<string[]>([]);
  let bccRecipients = $state<string[]>([]);
  let showCc = $state(false);
  let showBcc = $state(false);
  let subject = $state(initialSubject);
  let body = $state(initialBody);
  let attachments = $state<{ filename: string; content_type: string; base64_content: string; size: number }[]>(initialAttachments);
  import { ChevronDown } from 'lucide-svelte';
  let fromAccount = $state('');
  let accounts = $state<Account[]>([]);
  let isSending = $state(false);
  let showAccountDropdown = $state(false);

  let previousFromAccount = $state('');
  let activeSignatureId = $state<string | null>(null);
  let lastInjectedSignatureContent = $state('');

  function toggleAccountDropdown() {
    showAccountDropdown = !showAccountDropdown;
  }

  function selectAccount(accountId: string) {
    fromAccount = accountId;
    showAccountDropdown = false;
  }

  $effect(() => {
    if (fromAccount && fromAccount !== previousFromAccount) {
      previousFromAccount = fromAccount;

      const defaultSig = templateStore.signatures.find(s => s.accountId === fromAccount && s.isDefault);
      if (defaultSig) {
        swapSignature(defaultSig.id);
      } else {
        removeSignature();
      }
    }
  });

  $effect(() => {
    if (isOpen) {
      loadAccounts();
    } else {
      // Reset form when closed
      toRecipients = [];
      ccRecipients = [];
      bccRecipients = [];
      subject = '';
      body = '';
      attachments = [];
    }
  });

  async function loadAccounts() {
    try {
      accounts = await apiClient.get('/api/v1/accounts');
      if (accounts.length > 0 && !fromAccount) {
        fromAccount = accounts[0].id;
      }
    } catch (e) {
      console.error("Failed to load accounts for compose modal", e);
    }
  }

  async function handleSend() {
    if (!fromAccount) {
      alert("Please select a sending account");
      return;
    }
    
    isSending = true;
    try {
      await apiClient.post('/api/v1/messages/send', {
        account_id: fromAccount,
        to: toRecipients,
        cc: ccRecipients.length > 0 ? ccRecipients : undefined,
        bcc: bccRecipients.length > 0 ? bccRecipients : undefined,
        subject,
        body_html: body.replace(/\n/g, '<br/>'),
        attachments: attachments.length > 0 ? attachments.map(a => ({
          filename: a.filename,
          content_type: a.content_type,
          base64_content: a.base64_content
        })) : undefined
      });
      
      onSend({
        to: toRecipients.join(', '),
        cc: ccRecipients.length > 0 ? ccRecipients.join(', ') : undefined,
        bcc: bccRecipients.length > 0 ? bccRecipients.join(', ') : undefined,
        subject,
        body
      });
      
      toRecipients = [];
      ccRecipients = [];
      bccRecipients = [];
      showCc = false;
      showBcc = false;
      subject = '';
      body = '';
      attachments = [];
      onClose();
    } catch (e: any) {
      alert(`Failed to send message: ${e.message}`);
    } finally {
      isSending = false;
    }
  }

  function applyFormat(prefix: string, suffix: string = '') {
    body = body ? `${body}\n${prefix}text${suffix}` : `${prefix}text${suffix}`;
  }

  let fileInput: HTMLInputElement;

  function handleAttach(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      for (let i = 0; i < target.files.length; i++) {
        const file = target.files[i];
        const reader = new FileReader();
        reader.onload = (event) => {
          if (event.target?.result) {
            attachments = [...attachments, {
              filename: file.name,
              content_type: file.type || 'application/octet-stream',
              base64_content: event.target.result as string,
              size: file.size
            }];
          }
        };
        reader.readAsDataURL(file);
      }
    }
    target.value = ''; // Reset input
  }

  function handleBodyInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    const currentBody = target.value;

    // Check the text right up to the cursor to see if it matches a snippet
    const cursorPosition = target.selectionStart;
    const textBeforeCursor = currentBody.substring(0, cursorPosition);
    const textAfterCursor = currentBody.substring(cursorPosition);

    for (const snippet of templateStore.snippets) {
      // Escape shortcut to avoid regex bugs if shortcut is e.g., '/.thanks'
      const escapedShortcut = snippet.shortcut.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      // Look for the shortcut followed by a space right at the cursor position
      const regex = new RegExp(escapedShortcut + '(\\s)$');
      if (regex.test(textBeforeCursor)) {
        let replacement = snippet.template;

        // Find variables like {{name}} and prompt for them
        const varRegex = /\{\{([^}]+)\}\}/g;
        let match;
        const variablesToReplace: { original: string, name: string }[] = [];
        while ((match = varRegex.exec(replacement)) !== null) {
          variablesToReplace.push({ original: match[0], name: match[1] });
        }

        for (const v of variablesToReplace) {
          const val = window.prompt(`Enter value for ${v.name}:`, '');
          if (val !== null) {
            replacement = replacement.replace(new RegExp(v.original.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'), val);
          }
        }

        // Rebuild body with the replacement
        body = textBeforeCursor.replace(regex, replacement + '$1') + textAfterCursor;

        // Use timeout to reset cursor position after svelte updates the DOM
        setTimeout(() => {
          const newPos = cursorPosition - snippet.shortcut.length + replacement.length;
          target.selectionStart = newPos;
          target.selectionEnd = newPos;
        }, 0);

        break; // Only expand one at a time
      }
    }
  }

  function removeSignature() {
    if (!activeSignatureId) return;

    const sigStart = '\n\n<!-- data-kestrel-signature: start -->\n';
    const sigEnd = '\n<!-- data-kestrel-signature: end -->';

    const startIndex = body.indexOf(sigStart);
    const endIndex = body.indexOf(sigEnd);

    if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
      const currentSignatureContent = body.substring(startIndex + sigStart.length, endIndex);

      if (currentSignatureContent !== lastInjectedSignatureContent) {
        if (!window.confirm("Your signature has been modified. Are you sure you want to change it?")) {
          return false;
        }
      }

      body = body.substring(0, startIndex) + body.substring(endIndex + sigEnd.length);
      activeSignatureId = null;
      lastInjectedSignatureContent = '';
      return true;
    }

    activeSignatureId = null;
    lastInjectedSignatureContent = '';
    return true;
  }

  function swapSignature(sigId: string) {
    const sig = templateStore.signatures.find(s => s.id === sigId);
    if (!sig) return;

    const sigStart = '\n\n<!-- data-kestrel-signature: start -->\n';
    const sigEnd = '\n<!-- data-kestrel-signature: end -->';

    if (activeSignatureId) {
      const startIndex = body.indexOf(sigStart);
      const endIndex = body.indexOf(sigEnd);

      if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
        const currentSignatureContent = body.substring(startIndex + sigStart.length, endIndex);

        if (currentSignatureContent !== lastInjectedSignatureContent) {
          if (!window.confirm("Your signature has been modified. Are you sure you want to change it?")) {
            return;
          }
        }

        body = body.substring(0, startIndex) + sigStart + sig.htmlContent + sigEnd + body.substring(endIndex + sigEnd.length);
        activeSignatureId = sig.id;
        lastInjectedSignatureContent = sig.htmlContent;
        return;
      }
    }

    // No existing signature block found or wasn't active
    body += sigStart + sig.htmlContent + sigEnd;
    activeSignatureId = sig.id;
    lastInjectedSignatureContent = sig.htmlContent;
  }

  function appendSignature(sigId: string) {
    swapSignature(sigId);
  }
</script>

{#if isOpen}
  <input type="file" bind:this={fileInput} onchange={handleAttach} class="hidden" multiple />
  <div transition:fade={{ duration: 200 }} class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
    <div transition:fly={{ y: 50, duration: 400, easing: cubicOut }} class="w-full max-w-2xl bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl flex flex-col overflow-hidden font-sans">
      <!-- Header -->
      <div class="px-4 py-3 border-b border-[var(--color-border-hairline)] flex items-center justify-between bg-[var(--color-canvas-card)]">
        <span class="text-xs font-medium text-[var(--color-text-secondary)] tracking-wide">New Message</span>
        <button onclick={onClose} class="p-1 rounded hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Form Inputs (Borderless, Dividers) -->
      <div class="text-xs">
        <!-- From Account Row -->
        <div class="flex items-center px-4 py-2 border-b border-[var(--color-border-hairline)]/30 relative">
          <span class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 select-none">From:</span>
          {#if accounts.length === 0}
            <div class="flex-1 text-[var(--color-text-primary)] py-0.5">Loading accounts...</div>
          {:else}
            <button
              type="button"
              class="flex-1 flex items-center justify-between bg-transparent text-[var(--color-text-primary)] outline-none border-none cursor-pointer py-0.5"
              onclick={toggleAccountDropdown}
            >
              <div class="flex items-center gap-2">
                {#if accounts.find(a => a.id === fromAccount)}
                  {@const activeAccount = accounts.find(a => a.id === fromAccount)}
                  {#if activeAccount}
                    <div class="w-4 h-4 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-[10px] font-bold">
                      {activeAccount.display_name.charAt(0).toUpperCase()}
                    </div>
                    <span>{activeAccount.display_name} &lt;{activeAccount.provider_account_id}&gt;</span>
                  {/if}
                {:else}
                  <span>Select account...</span>
                {/if}
              </div>
              <ChevronDown class="w-4 h-4 text-[var(--color-text-secondary)]" />
            </button>

            {#if showAccountDropdown}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="fixed inset-0 z-40" onclick={() => showAccountDropdown = false}></div>
              <div class="absolute left-14 top-full mt-1 w-80 bg-[var(--color-canvas-elevated)] border border-[var(--color-border-hairline)] rounded-lg shadow-xl z-50 overflow-hidden text-xs">
                {#each accounts as account}
                  <button
                    type="button"
                    class="w-full text-left px-3 py-2 flex items-center gap-3 hover:bg-white/5 transition-colors {fromAccount === account.id ? 'bg-white/5' : ''}"
                    onclick={() => selectAccount(account.id)}
                  >
                    <div class="w-5 h-5 shrink-0 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-[10px] font-bold">
                      {account.display_name.charAt(0).toUpperCase()}
                    </div>
                    <div class="flex-1 truncate">
                      <div class="font-medium text-[var(--color-text-primary)] truncate">{account.display_name}</div>
                      <div class="text-[10px] text-[var(--color-text-secondary)] truncate">{account.provider_account_id}</div>
                    </div>
                  </button>
                {/each}
              </div>
            {/if}
          {/if}
        </div>

        <!-- To Row -->
        <div class="flex items-center px-4 py-1.5 border-b border-[var(--color-border-hairline)]/30">
          <span class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 select-none">To:</span>
          <div class="flex-1 flex items-center">
            <div class="flex-1">
              <ContactAutocomplete bind:recipients={toRecipients} placeholder="Recipients..." />
            </div>
            <div class="flex items-center gap-2 ml-2 text-[11px] text-[var(--color-text-secondary)] shrink-0 select-none">
              {#if !showCc}
                <button type="button" onclick={() => showCc = true} class="hover:text-white transition-colors cursor-pointer font-mono">Cc</button>
              {/if}
              {#if !showBcc}
                <button type="button" onclick={() => showBcc = true} class="hover:text-white transition-colors cursor-pointer font-mono">Bcc</button>
              {/if}
            </div>
          </div>
        </div>

        <!-- Cc Row -->
        {#if showCc}
          <div class="flex items-center px-4 py-1.5 border-b border-[var(--color-border-hairline)]/30">
            <div class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 flex items-center justify-between select-none">
              <span>Cc:</span>
              <button type="button" onclick={() => { showCc = false; ccRecipients = []; }} class="text-[10px] text-[var(--color-text-secondary)] hover:text-white mr-1.5 cursor-pointer">✕</button>
            </div>
            <div class="flex-1">
              <ContactAutocomplete bind:recipients={ccRecipients} placeholder="Cc recipients..." />
            </div>
          </div>
        {/if}

        <!-- Bcc Row -->
        {#if showBcc}
          <div class="flex items-center px-4 py-1.5 border-b border-[var(--color-border-hairline)]/30">
            <div class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 flex items-center justify-between select-none">
              <span>Bcc:</span>
              <button type="button" onclick={() => { showBcc = false; bccRecipients = []; }} class="text-[10px] text-[var(--color-text-secondary)] hover:text-white mr-1.5 cursor-pointer">✕</button>
            </div>
            <div class="flex-1">
              <ContactAutocomplete bind:recipients={bccRecipients} placeholder="Bcc recipients..." />
            </div>
          </div>
        {/if}

        <!-- Subject Field (Borderless, No label) -->
        <div class="px-4 py-2 border-b border-[var(--color-border-hairline)]/30">
          <input
            type="text"
            bind:value={subject}
            placeholder="Subject"
            class="w-full bg-transparent text-sm font-medium text-[var(--color-text-primary)] placeholder-[var(--color-text-secondary)]/60 outline-none border-none"
          />
        </div>
        
        <!-- Attachments Row -->
        {#if attachments.length > 0}
          <div class="px-4 py-2 border-b border-[var(--color-border-hairline)]/30 flex gap-2 flex-wrap bg-[var(--color-canvas-base)]/30">
            {#each attachments as att, index}
              <div class="flex items-center gap-1.5 bg-[var(--color-border-hairline)]/40 rounded px-2.5 py-1 text-[11px] text-[var(--color-text-primary)] font-medium">
                <Paperclip class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
                <span>{att.filename}</span>
                <span class="text-[var(--color-text-secondary)]/60 font-mono text-[9px]">{(att.size / 1024).toFixed(0)}KB</span>
                <button type="button" onclick={() => attachments = attachments.filter((_, i) => i !== index)} class="ml-1 hover:text-white text-[var(--color-text-secondary)]"><X class="w-3 h-3" /></button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Message Textarea Body -->
      <div class="p-4 flex-1">
        <textarea
          bind:value={body}
          oninput={handleBodyInput}
          placeholder="Write your email response or message here..."
          rows="10"
          class="w-full h-full bg-transparent text-[var(--color-text-primary)] text-xs outline-none resize-none leading-relaxed border-none"
        ></textarea>
      </div>

      <!-- Footer Actions & Style Controls -->
      <div class="px-4 py-3 border-t border-[var(--color-border-hairline)] flex items-center justify-between bg-[var(--color-canvas-card)]">
        <div class="flex items-center gap-2">
          <button disabled={isSending} onclick={handleSend} class="px-4 py-1.5 rounded bg-white text-black text-xs font-semibold hover:bg-neutral-200 transition-all flex items-center gap-1.5 cursor-pointer shadow-sm active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed">
            <Send class="w-3.5 h-3.5" />
            <span>{isSending ? 'Sending...' : 'Send Message'}</span>
          </button>

          <!-- Formatting Style Controls -->
          <div class="flex items-center gap-0.5 ml-2 border-l border-[var(--color-border-hairline)]/40 pl-2 text-[var(--color-text-secondary)]">
            <button onclick={() => applyFormat('**', '**')} title="Bold" class="p-1.5 rounded hover:bg-white/10 hover:text-white transition-colors cursor-pointer"><Bold class="w-3.5 h-3.5" /></button>
            <button onclick={() => applyFormat('*', '*')} title="Italic" class="p-1.5 rounded hover:bg-white/10 hover:text-white transition-colors cursor-pointer"><Italic class="w-3.5 h-3.5" /></button>
            <button onclick={() => applyFormat('- ')} title="List" class="p-1.5 rounded hover:bg-white/10 hover:text-white transition-colors cursor-pointer"><List class="w-3.5 h-3.5" /></button>
            <button onclick={() => applyFormat('[', '](https://)')} title="Link" class="p-1.5 rounded hover:bg-white/10 hover:text-white transition-colors cursor-pointer"><Link class="w-3.5 h-3.5" /></button>
            <button onclick={() => fileInput.click()} title="Attach File" class="p-1.5 rounded hover:bg-white/10 hover:text-white transition-colors cursor-pointer"><Paperclip class="w-3.5 h-3.5" /></button>

            {#if templateStore.signatures.length > 0}
              <div class="relative ml-2 flex items-center">
                <select
                  class="appearance-none bg-transparent outline-none border-none text-[10px] text-[var(--color-text-secondary)] hover:text-white cursor-pointer"
                  onchange={(e) => {
                    const val = e.currentTarget.value;
                    if (val === 'none') {
                      removeSignature();
                    } else if (val) {
                      swapSignature(val);
                    }
                    e.currentTarget.value = '';
                  }}
                >
                  <option value="" disabled selected>Signatures...</option>
                  {#each templateStore.signatures as sig}
                    <option value={sig.id} class="bg-[var(--color-canvas-card)] text-[var(--color-text-primary)]">{sig.name}</option>
                  {/each}
                  <option value="none" class="bg-[var(--color-canvas-card)] text-red-400">Remove Signature</option>
                </select>
              </div>
            {/if}
          </div>
        </div>

        <button onclick={onClose} class="px-3 py-1.5 rounded text-xs text-[var(--color-text-secondary)] hover:text-white cursor-pointer transition-colors" title="Discard draft">
          Discard Draft
        </button>
      </div>
    </div>
  </div>
{/if}
