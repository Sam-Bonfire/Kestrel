<script lang="ts">
  import { X, Send, Paperclip, Bold, Italic, List, Link } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { EmailPillInput } from '@kestrel/shared/components';

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
  let fromAccount = $state('user@kestrel.dev');

  function handleSend() {
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
    onClose();
  }

  function applyFormat(prefix: string, suffix: string = '') {
    body = body ? `${body}\n${prefix}text${suffix}` : `${prefix}text${suffix}`;
  }

  let fileInput: HTMLInputElement;

  function handleAttach(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      const fileName = target.files[0].name;
      body = body ? `${body}\n\n[Attached: ${fileName}]` : `[Attached: ${fileName}]`;
    }
  }
</script>

{#if isOpen}
  <input type="file" bind:this={fileInput} onchange={handleAttach} class="hidden" />
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
            <option value="user@kestrel.dev" class="bg-[var(--color-canvas-card)]">user@kestrel.dev (Default)</option>
            <option value="sam@outlook.com" class="bg-[var(--color-canvas-card)]">sam@outlook.com</option>
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
          <button onclick={handleSend} class="px-4 py-1.5 rounded bg-white text-black text-xs font-semibold hover:bg-neutral-200 transition-all flex items-center gap-1.5 cursor-pointer shadow-sm active:scale-95">
            <Send class="w-3.5 h-3.5" />
            <span>Send Message</span>
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
