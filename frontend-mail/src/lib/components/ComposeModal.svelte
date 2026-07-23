<script lang="ts">
  import { X, Send, Paperclip, Bold, Italic, List, Link } from 'lucide-svelte';
  import { EmailPillInput } from '@kestrel/shared/components';

  let {
    isOpen = false,
    onClose = () => {},
    onSend = (draft: { to: string; subject: string; body: string }) => {}
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSend?: (draft: { to: string; subject: string; body: string }) => void;
  }>();

  let toRecipients = $state<string[]>([]);
  let subject = $state('');
  let body = $state('');
  let fromAccount = $state('user@kestrel.dev');

  function handleSend() {
    onSend({ to: toRecipients.join(', '), subject, body });
    toRecipients = [];
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
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
    <div class="w-full max-w-2xl bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl flex flex-col overflow-hidden font-sans">
      <!-- Header -->
      <div class="px-4 py-3 border-b border-[var(--color-border-hairline)] flex items-center justify-between bg-[var(--color-canvas-base)]">
        <h3 class="text-xs font-semibold text-white uppercase tracking-wider">New Message</h3>
        <button onclick={onClose} class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Form Inputs -->
      <div class="p-4 space-y-3 border-b border-[var(--color-border-hairline)]/40 text-xs">
        <div class="flex items-center gap-2">
          <span class="w-12 text-[var(--color-text-secondary)] font-mono">From:</span>
          <select bind:value={fromAccount} class="flex-1 bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] border border-[var(--color-border-hairline)] rounded px-2.5 py-1 outline-none">
            <option value="user@kestrel.dev">user@kestrel.dev (Default)</option>
            <option value="sam@outlook.com">sam@outlook.com</option>
          </select>
        </div>

        <div class="flex items-center gap-2">
          <span class="w-12 text-[var(--color-text-secondary)] font-mono shrink-0">To:</span>
          <div class="flex-1 bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] border border-[var(--color-border-hairline)] rounded px-2.5 py-0.5 focus-within:border-white/30 transition-colors">
            <EmailPillInput bind:recipients={toRecipients} placeholder="recipient@example.com" />
          </div>
        </div>

        <div class="flex items-center gap-2">
          <span class="w-12 text-[var(--color-text-secondary)] font-mono">Subject:</span>
          <input
            type="text"
            bind:value={subject}
            placeholder="Subject line..."
            class="flex-1 bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] border border-[var(--color-border-hairline)] rounded px-2.5 py-1 outline-none focus:border-white/30"
          />
        </div>
      </div>

      <!-- Editor Controls Toolbar -->
      <div class="px-4 py-2 bg-[var(--color-canvas-base)] border-b border-[var(--color-border-hairline)]/40 flex items-center gap-2 text-[var(--color-text-secondary)]">
        <button onclick={() => applyFormat('**', '**')} title="Bold" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors cursor-pointer"><Bold class="w-3.5 h-3.5" /></button>
        <button onclick={() => applyFormat('*', '*')} title="Italic" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors cursor-pointer"><Italic class="w-3.5 h-3.5" /></button>
        <button onclick={() => applyFormat('- ')} title="List" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors cursor-pointer"><List class="w-3.5 h-3.5" /></button>
        <button onclick={() => applyFormat('[', '](https://)')} title="Link" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors cursor-pointer"><Link class="w-3.5 h-3.5" /></button>
        <button onclick={() => fileInput.click()} title="Attach File" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors cursor-pointer"><Paperclip class="w-3.5 h-3.5" /></button>
      </div>

      <!-- Message Textarea Body -->
      <div class="p-4 flex-1">
        <textarea
          bind:value={body}
          placeholder="Write your email response or message here..."
          rows="10"
          class="w-full h-full bg-transparent text-[var(--color-text-primary)] text-xs outline-none resize-none leading-relaxed"
        ></textarea>
      </div>

      <!-- Footer Actions -->
      <div class="px-4 py-3 border-t border-[var(--color-border-hairline)] flex items-center justify-between bg-[var(--color-canvas-base)]">
        <button onclick={onClose} class="px-3 py-1.5 rounded text-xs text-[var(--color-text-secondary)] hover:text-white">
          Discard Draft
        </button>

        <button onclick={handleSend} class="px-4 py-1.5 rounded bg-white text-black text-xs font-semibold hover:bg-neutral-200 transition-all flex items-center gap-1.5">
          <Send class="w-3.5 h-3.5" />
          <span>Send Message</span>
        </button>
      </div>
    </div>
  </div>
{/if}
