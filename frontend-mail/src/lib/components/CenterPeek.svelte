<script lang="ts">
  import { Avatar } from '@kestrel/shared';
  import {
    X,
    ChevronUp,
    ChevronDown,
    Star,
    Archive,
    Trash2,
    Mail,
    Reply,
    ReplyAll,
    Forward,
    Plus,
    CornerUpLeft,
    Send,
    Sparkles,
    Check,
    Clock,
    FileText,
    MoreVertical,
    Tag,
    Paperclip
  } from 'lucide-svelte';
  import { 
    labelCustomizations, 
    getLabelStyle,
    EmailPillInput
  } from '@kestrel/shared';

  export interface Email {
    id: string;
    sender: string;
    senderEmail: string;
    to: string;
    subject: string;
    body: string;
    timestamp: string;
    isUnread: boolean;
    isStarred: boolean;
    isArchived: boolean;
    isTrash: boolean;
    labels: string[];
    avatar?: string;
  }

  let {
    email = null,
    onClose = () => {},
    onNavigate = (dir: 'prev' | 'next') => {},
    hasPrev = false,
    hasNext = false,
    onArchive = (id: string) => {},
    onDelete = (id: string) => {},
    onToggleStar = (id: string) => {},
    onToggleUnread = (id: string) => {},
    onAddLabel = (id: string, label: string) => {},
    onRemoveLabel = (id: string, label: string) => {},
    onSendReply = (emailId: string, replyBody: string) => {},
    historicalMessages = [] as { sender: string; body: string; timestamp: string }[]
  } = $props<{
    email?: Email | null;
    onClose?: () => void;
    onNavigate?: (direction: 'prev' | 'next') => void;
    hasPrev?: boolean;
    hasNext?: boolean;
    onArchive?: (id: string) => void;
    onDelete?: (id: string) => void;
    onToggleStar?: (id: string) => void;
    onToggleUnread?: (id: string) => void;
    onAddLabel?: (id: string, label: string) => void;
    onRemoveLabel?: (id: string, label: string) => void;
    onSendReply?: (emailId: string, replyBody: string) => void;
    historicalMessages?: { sender: string; body: string; timestamp: string }[];
  }>();

  // UI state variables
  let showReplyDraft = $state(false);
  let replyType = $state<'reply' | 'reply_all' | 'forward'>('reply');
  let replyToRecipients = $state<string[]>([]);
  let replyText = $state('');
  let newLabelText = $state('');
  let showAddLabelInput = $state(false);

  // AI Summary panel state
  let isAiSummaryLoading = $state(false);
  let aiSummaryText = $state<string | null>(null);
  let showAiSummary = $state(false);
  
  let showHistory = $state(false);

  function handleSendReply() {
    if (replyText.trim() && email) {
      onSendReply(email.id, replyText.trim());
      replyText = '';
      replyToRecipients = [];
      showReplyDraft = false;
    }
  }

  function handleAddLabelSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (newLabelText.trim() && email) {
      onAddLabel(email.id, newLabelText.trim());
      newLabelText = '';
      showAddLabelInput = false;
    }
  }

  function handleTriggerAiSummary() {
    if (showAiSummary) {
      showAiSummary = false;
      return;
    }

    isAiSummaryLoading = true;
    showAiSummary = true;
    aiSummaryText = null;

    setTimeout(() => {
      isAiSummaryLoading = false;
      aiSummaryText = "Kestrel AI Summary:\n• Action required: Review and upgrade vulnerable node package dependencies immediately.\n• Primary focus: High vulnerability issue found in frontend/backend libraries.\n• Owner: Alex Rivera.";
    }, 1200);
  }
</script>

{#if email}
  <div
    id="center-peek-overlay"
    class="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4 bg-black/50 backdrop-blur-[2px]"
    role="button"
    tabindex="0"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <!-- Modal Container -->
    <div
      id="center-peek-modal"
      class="w-full md:max-w-4xl h-screen md:h-[90vh] bg-[#0d0d0d] flex flex-col rounded-none md:rounded-xl shadow-2xl overflow-hidden font-sans border border-[var(--color-border-hairline)]"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header Toolbar -->
      <div 
        id="peek-header" 
        class="flex px-4 py-2 bg-[var(--color-canvas-base)] items-center justify-between select-none shrink-0 cursor-default relative"
      >
        <!-- Transparent drag handle that stops before WindowControls -->
        <div class="absolute inset-y-0 left-0 right-36" data-tauri-drag-region></div>
        <!-- Left tools -->
        <div class="flex items-center gap-1">
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer disabled:opacity-30" disabled={!hasPrev} onclick={() => onNavigate('prev')} title="Previous Email">
            <ChevronUp class="w-4 h-4" />
          </button>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer disabled:opacity-30" disabled={!hasNext} onclick={() => onNavigate('next')} title="Next Email">
            <ChevronDown class="w-4 h-4" />
          </button>
        </div>

        <!-- AI Magic & Actions toolbar -->
        <div class="flex items-center gap-1.5">
          <button
            onclick={handleTriggerAiSummary}
            class="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 border border-blue-500/10 transition-colors cursor-pointer mr-2"
          >
            <Sparkles class="w-3.5 h-3.5 text-blue-400" />
            <span>AI Summary</span>
          </button>
          
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => { onToggleUnread(email!.id); onClose(); }} title="Mark as Unread">
            <Mail class="w-4 h-4 text-[var(--color-text-secondary)]" />
          </button>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => { onArchive(email!.id); onClose(); }} title="Archive Email (e)">
            <Archive class="w-4 h-4 text-[var(--color-text-secondary)]" />
          </button>
          <button class="p-1.5 rounded-lg hover:bg-red-500/20 text-red-400 transition-colors cursor-pointer" onclick={() => { onDelete(email!.id); onClose(); }} title="Delete Email">
            <Trash2 class="w-4 h-4" />
          </button>
          
          <div class="w-px h-5 bg-[var(--color-border-hairline)] mx-1"></div>

          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => window.print()} title="Print">
            <FileText class="w-4 h-4 text-[var(--color-text-secondary)]" />
          </button>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => {}} title="More actions">
            <MoreVertical class="w-4 h-4 text-[var(--color-text-secondary)]" />
          </button>
          <div class="w-px h-5 bg-[var(--color-border-hairline)] mx-1"></div>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={onClose} title="Close peek (Esc)">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Scrollable Body -->
      <div id="peek-scrollable-body" class="flex-1 overflow-y-auto px-6 md:px-8 py-6 space-y-6 bg-[var(--color-canvas-base)]">
        
        <!-- Subject & Labels Bar -->
        <div class="space-y-2">
          <h1 class="text-xl md:text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">
            {email.subject}
          </h1>

          <!-- Correctly visible and editable labels bar using customizations -->
          <div id="peek-labels-bar" class="flex flex-wrap items-center gap-1.5">
            {#each email.labels as label}
              {@const labelStyle = getLabelStyle(label, $labelCustomizations)}
              <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-medium border {labelStyle.bgColor} {labelStyle.textColor} {labelStyle.borderColor} animate-fadeIn">
                <span>{label.split('/').pop()}</span>
                <button
                  onclick={() => onRemoveLabel(email!.id, label)}
                  class="hover:bg-white/10 p-0.5 rounded-full text-current cursor-pointer transition-colors"
                  title="Remove label"
                >
                  <X class="w-2.5 h-2.5" />
                </button>
              </span>
            {/each}

            {#if showAddLabelInput}
              <form onsubmit={handleAddLabelSubmit} class="flex items-center gap-1.5">
                <input
                  type="text"
                  placeholder="Tag name..."
                  bind:value={newLabelText}
                  class="bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded px-2.5 py-1 text-[11px] text-white outline-none font-mono focus:border-white/20 transition-all"
                />
                <button type="submit" class="px-3 py-1 rounded bg-blue-500 text-white text-xs font-semibold hover:bg-blue-600 cursor-pointer">Add</button>
                <button type="button" class="p-1 rounded hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white cursor-pointer" onclick={() => showAddLabelInput = false}><X class="w-3 h-3" /></button>
              </form>
            {:else}
              <button type="button" class="p-1 rounded hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => showAddLabelInput = true} title="Add Label">
                <Plus class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>
        </div>

        <!-- AI Summary expanded card -->
        {#if showAiSummary}
          <div class="p-4 bg-blue-500/5 border border-blue-500/10 rounded-xl space-y-2 animate-slideDown">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2 text-xs font-semibold text-blue-400">
                <Sparkles class="w-3.5 h-3.5" />
                <span>Kestrel AI Assistant</span>
              </div>
              <button onclick={() => showAiSummary = false} class="text-[var(--color-text-secondary)] hover:text-white transition-colors">
                <X class="w-3.5 h-3.5" />
              </button>
            </div>
            
            {#if isAiSummaryLoading}
              <div class="flex items-center gap-2 py-2 text-xs text-[var(--color-text-secondary)]">
                <span class="w-1.5 h-1.5 rounded-full bg-blue-400 animate-ping"></span>
                <span>Generating digest summary...</span>
              </div>
            {:else}
              <pre class="text-xs text-[var(--color-text-primary)] font-sans whitespace-pre-wrap leading-relaxed">
                {aiSummaryText}
              </pre>
            {/if}
          </div>
        {/if}

        <!-- Message Thread Stack using shared Reusable Avatar component -->
        {#if historicalMessages && historicalMessages.length > 0}
          <div class="space-y-3.5 border-b border-[var(--color-border-hairline)]/30 pb-4">
            {#if !showHistory}
              <button 
                onclick={() => showHistory = true}
                class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] text-xs text-[var(--color-text-secondary)] hover:text-white transition-colors mx-auto cursor-pointer"
              >
                <MoreVertical class="w-3.5 h-3.5" />
                <span>Show {historicalMessages.length} earlier messages</span>
              </button>
            {:else}
              <div class="flex items-center justify-between">
                <span class="text-[10px] font-mono tracking-widest text-[var(--color-text-secondary)]/50 uppercase">Conversation Thread</span>
                <button onclick={() => showHistory = false} class="text-xs text-blue-400 hover:underline cursor-pointer">Hide</button>
              </div>
              {#each historicalMessages as msg}
                <div class="flex gap-3 items-start border border-[var(--color-border-hairline)]/20 rounded-xl p-3.5 bg-[#131313]/20">
                  <Avatar name={msg.sender} size={28} />
                  <div class="space-y-1 min-w-0 flex-1">
                    <div class="flex items-baseline justify-between">
                      <span class="text-xs font-semibold text-white">{msg.sender}</span>
                      <span class="text-[9px] font-mono text-[var(--color-text-secondary)]">{msg.timestamp}</span>
                    </div>
                    <p class="text-xs text-[var(--color-text-primary)] leading-relaxed">{@html msg.body}</p>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        {/if}

        <!-- Primary Message Body -->
        <div class="space-y-4">
          <!-- Sender Card -->
          <div class="flex items-center justify-between bg-[#131313] border border-[var(--color-border-hairline)] rounded-xl p-4 gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <Avatar name={email.sender} size={32} />
              <div class="space-y-0.5 min-w-0">
                <div class="flex items-center gap-1.5 text-sm min-w-0">
                  <span class="font-semibold text-white truncate">{email.sender}</span>
                  <span class="text-[var(--color-text-secondary)] text-xs font-mono truncate">&lt;{email.senderEmail}&gt;</span>
                </div>
                <div class="text-xs text-[var(--color-text-secondary)] truncate">
                  To: <span class="font-mono">{email.to}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-3 shrink-0">
              <span class="text-xs font-mono text-[var(--color-text-secondary)] hidden sm:block">
                {email.timestamp}
              </span>
              <button 
                onclick={(e) => { e.stopPropagation(); onToggleStar(email!.id); }}
                class="p-1.5 rounded-full hover:bg-white/5 transition-colors cursor-pointer"
                title={email.isStarred ? "Remove Star" : "Star Email"}
              >
                <Star class="w-4 h-4 {email.isStarred ? 'fill-current text-amber-400' : 'text-[var(--color-text-secondary)]'}" />
              </button>
              <button class="p-1.5 rounded-full hover:bg-white/5 text-[var(--color-text-secondary)] transition-colors cursor-pointer">
                <Reply class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Message HTML Render Content -->
          <div class="prose prose-invert max-w-none text-sm text-[var(--color-text-primary)] font-sans leading-[1.7] border border-[var(--color-border-hairline)]/30 rounded-xl p-5 bg-[#131313]/10">
            {@html email.body}
          </div>

          <!-- Attachment Mock -->
          <div class="flex gap-3 overflow-x-auto pb-2">
            <div class="flex items-center gap-3 px-3 py-2 bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg shrink-0 cursor-pointer hover:bg-[var(--color-canvas-hover)] transition-colors">
              <div class="w-8 h-8 rounded bg-blue-500/10 flex items-center justify-center text-blue-400">
                <FileText class="w-4 h-4" />
              </div>
              <div class="flex flex-col">
                <span class="text-xs font-semibold text-white">Project_Brief_v2.pdf</span>
                <span class="text-[10px] text-[var(--color-text-secondary)]">2.4 MB</span>
              </div>
            </div>
            <div class="flex items-center gap-3 px-3 py-2 bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg shrink-0 cursor-pointer hover:bg-[var(--color-canvas-hover)] transition-colors">
              <div class="w-8 h-8 rounded bg-purple-500/10 flex items-center justify-center text-purple-400">
                <Paperclip class="w-4 h-4" />
              </div>
              <div class="flex flex-col">
                <span class="text-xs font-semibold text-white">budget_q3.xlsx</span>
                <span class="text-[10px] text-[var(--color-text-secondary)]">1.1 MB</span>
              </div>
            </div>
          </div>
        </div>



        <!-- Action Buttons / Inline Reply Editor -->
        {#if !showReplyDraft}
          <div class="flex items-center gap-3 pt-2">
            <button
              onclick={() => { 
                replyType = 'reply'; 
                replyToRecipients = email.senderEmail ? [email.senderEmail] : [email.sender]; 
                showReplyDraft = true; 
              }}
              class="flex items-center gap-2 px-4 py-2.5 bg-[var(--color-canvas-card)] hover:bg-[var(--color-canvas-hover)] text-white text-xs font-medium rounded-xl border border-[var(--color-border-hairline)] transition-all cursor-pointer shadow-sm hover:border-white/20 active:scale-95"
            >
              <Reply class="w-4 h-4 text-blue-400" />
              <span>Reply</span>
            </button>

            <button
              onclick={() => { 
                replyType = 'reply_all'; 
                replyToRecipients = [email.senderEmail || email.sender, 'alex@kestrel.dev']; 
                showReplyDraft = true; 
              }}
              class="flex items-center gap-2 px-4 py-2.5 bg-[var(--color-canvas-card)] hover:bg-[var(--color-canvas-hover)] text-white text-xs font-medium rounded-xl border border-[var(--color-border-hairline)] transition-all cursor-pointer shadow-sm hover:border-white/20 active:scale-95"
            >
              <ReplyAll class="w-4 h-4 text-indigo-400" />
              <span>Reply All</span>
            </button>

            <button
              onclick={() => { 
                replyType = 'forward'; 
                replyToRecipients = []; 
                showReplyDraft = true; 
              }}
              class="flex items-center gap-2 px-4 py-2.5 bg-[var(--color-canvas-card)] hover:bg-[var(--color-canvas-hover)] text-white text-xs font-medium rounded-xl border border-[var(--color-border-hairline)] transition-all cursor-pointer shadow-sm hover:border-white/20 active:scale-95"
            >
              <Forward class="w-4 h-4 text-purple-400" />
              <span>Forward</span>
            </button>
          </div>
        {:else}
          <!-- Expanded rich editor -->
          <div class="border border-[var(--color-border-hairline)] rounded-xl overflow-hidden bg-[#131313] p-4 space-y-3 animate-fadeIn">
            <div class="flex items-center justify-between border-b border-[var(--color-border-hairline)]/60 pb-2">
              <div class="flex items-center gap-2 text-xs font-semibold text-white">
                {#if replyType === 'reply'}
                  <Reply class="w-3.5 h-3.5 text-blue-400" />
                  <span>Replying</span>
                {:else if replyType === 'reply_all'}
                  <ReplyAll class="w-3.5 h-3.5 text-indigo-400" />
                  <span>Replying All</span>
                {:else}
                  <Forward class="w-3.5 h-3.5 text-purple-400" />
                  <span>Forwarding Message</span>
                {/if}
              </div>
              <button 
                onclick={() => showReplyDraft = false}
                class="text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <!-- Editable Recipients Input Field with Email Pills -->
            <div class="flex items-center gap-2 px-3 py-1 bg-[var(--color-canvas-base)] rounded-lg border border-[var(--color-border-hairline)] text-xs focus-within:border-white/30 transition-colors">
              <span class="text-[var(--color-text-secondary)] font-mono text-[11px] w-6 shrink-0">To:</span>
              <EmailPillInput bind:recipients={replyToRecipients} placeholder="recipient@example.com" />
            </div>

            <textarea
              bind:value={replyText}
              placeholder={replyType === 'forward' ? "Add forwarding note..." : "Write your response..."}
              rows="5"
              class="w-full bg-[var(--color-canvas-base)] rounded-lg text-xs text-white outline-none border border-[var(--color-border-hairline)] resize-none font-sans p-3 focus:border-blue-500/50 transition-colors"
            ></textarea>

            <div class="flex justify-between items-center pt-2">
              <div class="flex items-center gap-1.5">
                <button onclick={() => replyText += '**text**'} class="p-1.5 rounded hover:bg-white/5 text-[var(--color-text-secondary)] transition-colors cursor-pointer" title="Bold"><strong class="font-serif font-bold">B</strong></button>
                <button onclick={() => replyText += '*text*'} class="p-1.5 rounded hover:bg-white/5 text-[var(--color-text-secondary)] transition-colors cursor-pointer" title="Italic"><em class="font-serif">I</em></button>
                <button onclick={() => replyText += '<u>text</u>'} class="p-1.5 rounded hover:bg-white/5 text-[var(--color-text-secondary)] transition-colors cursor-pointer" title="Underline"><span class="underline">U</span></button>
              </div>
              <div class="flex items-center gap-2">
                <button onclick={() => showReplyDraft = false} class="px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer">Discard</button>
                <button
                  onclick={handleSendReply}
                  class="flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold bg-white text-black hover:bg-neutral-200 rounded-lg transition-all active:scale-95 cursor-pointer"
                >
                  <Send class="w-3.5 h-3.5" />
                  <span>{replyType === 'forward' ? 'Forward Email' : 'Send Reply'}</span>
                </button>
              </div>
            </div>
          </div>
        {/if}

      </div>
    </div>
  </div>
{/if}
