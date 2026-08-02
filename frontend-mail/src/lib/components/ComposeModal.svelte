<script lang="ts">
  import { X, Send, Paperclip, Bold, Italic, List, Link } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { EmailPillInput } from '@kestrel/shared/components';
  import { apiClient } from '@kestrel/shared/api/client';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { mailSignature } from '@kestrel/shared/stores/settings';

  let {
    isOpen = false,
    onClose = () => {},
    onSend = (draft: { to: string; cc?: string; bcc?: string; subject: string; body: string }) => {}
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSend?: (draft: { to: string; cc?: string; bcc?: string; subject: string; body: string }) => void;
  }>();

  let toRecipients = $state<string[]>([]);
  let ccRecipients = $state<string[]>([]);
  let bccRecipients = $state<string[]>([]);
  let showCc = $state(false);
  let showBcc = $state(false);
  let subject = $state('');
  let body = $state('');
  let attachments = $state<{ filename: string; content_type: string; base64_content: string; size: number }[]>([]);
  let fromAccount = $state('');
  let accounts = $state<any[]>([]);
  let isSending = $state(false);

  $effect(() => {
    if (isOpen) {
      loadAccounts();
      const sig = get(mailSignature);
      if (sig && body === '') {
        body = `<br><br>--<br>${sig}`;
      }
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
        <div class="flex items-center px-4 py-2 border-b border-[var(--color-border-hairline)]/30">
          <span class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 select-none">From:</span>
          <select bind:value={fromAccount} class="flex-1 bg-transparent text-[var(--color-text-primary)] outline-none border-none cursor-pointer py-0.5">
            {#if accounts.length === 0}
              <option value="" class="bg-[var(--color-canvas-card)]">Loading accounts...</option>
            {/if}
            {#each accounts as account}
              <option value={account.id} class="bg-[var(--color-canvas-card)]">{account.email_address} ({account.provider})</option>
            {/each}
          </select>
        </div>

        <!-- To Row -->
        <div class="flex items-center px-4 py-1.5 border-b border-[var(--color-border-hairline)]/30">
          <span class="w-10 text-[var(--color-text-secondary)] font-mono shrink-0 select-none">To:</span>
          <div class="flex-1 flex items-center">
            <div class="flex-1">
              <EmailPillInput bind:recipients={toRecipients} placeholder="Recipients..." />
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
              <EmailPillInput bind:recipients={ccRecipients} placeholder="Cc recipients..." />
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
              <EmailPillInput bind:recipients={bccRecipients} placeholder="Bcc recipients..." />
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
          </div>
        </div>

        <button onclick={onClose} class="px-3 py-1.5 rounded text-xs text-[var(--color-text-secondary)] hover:text-white cursor-pointer transition-colors" title="Discard draft">
          Discard Draft
        </button>
      </div>
    </div>
  </div>
{/if}
