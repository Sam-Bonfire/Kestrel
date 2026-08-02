<script lang="ts">
  import { Avatar } from '@kestrel/shared';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import {
    X,
    ChevronUp,
    ChevronDown,
    Star,
    Archive,
    ArchiveRestore,
    Trash2,
    Mail,
    MailOpen,
    Reply,
    ReplyAll,
    Forward,
    Plus,
    CornerUpLeft,
    Send,
    Clock,
    FileText,
    Tag,
    Paperclip,
    AlertOctagon,
    MoreVertical,
    Folder,
    Check,
    BellOff,
    ShieldAlert,
    UserX,
    CalendarPlus,
    Filter,
    Download
  } from 'lucide-svelte';
  import { 
    labelCustomizations, 
    getLabelStyle,
    EmailPillInput,
    Dropdown
  } from '@kestrel/shared';
  import DOMPurify from 'dompurify';

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
    isTrash: boolean;
    labels: string[];
    avatar?: string;
    attachments?: { filename: string; size: number }[];
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
    onSnooze = (id: string, until: string) => {},
    onMoveTo = (id: string, folder: string) => {},
    onReportSpam = (id: string) => {},
    onMute = (id: string) => {},
    onReportPhishing = (id: string) => {},
    onBlockSender = (emailAddress: string) => {},
    onCreateEvent = (id: string) => {},
    onFilterMessages = (emailAddress: string) => {},
    onDownloadMessage = (id: string) => {},
    historicalMessages = [] as { sender: string; body: string; timestamp: string }[],
    allLabels = [] as string[],
    initialReplyMode = null
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
    onSnooze?: (id: string, until: string) => void;
    onMoveTo?: (id: string, folder: string) => void;
    onReportSpam?: (id: string) => void;
    onMute?: (id: string) => void;
    onReportPhishing?: (id: string) => void;
    onBlockSender?: (emailAddress: string) => void;
    onCreateEvent?: (id: string) => void;
    onFilterMessages?: (emailAddress: string) => void;
    onDownloadMessage?: (id: string) => void;
    historicalMessages?: { sender: string; body: string; timestamp: string }[];
    allLabels?: string[];
    initialReplyMode?: 'reply' | 'reply_all' | 'forward' | null;
  }>();

  // UI state variables
  let showReplyDraft = $state(false);
  let replyType = $state<'reply' | 'reply_all' | 'forward'>('reply');
  let replyToRecipients = $state<string[]>([]);
  let replyText = $state('');
  let newLabelText = $state('');
  let showAddLabelInput = $state(false);
  let labelSearchQuery = $state('');
  let moveSearchQuery = $state('');

  let activeMenu = $state<'snooze' | 'label' | 'more' | 'move' | null>(null);

  // Dynamic more options
  const moreOptions = [
    { id: 'print', icon: FileText, label: 'Print', action: () => window.print() }
  ];

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

  $effect(() => {
    if (initialReplyMode && !showReplyDraft) {
      showReplyDraft = true;
      replyType = initialReplyMode;
    }
  });

</script>

{#if email}
  <div
    transition:fade={{ duration: 200 }}
    id="center-peek-overlay"
    class="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4 bg-black/50 backdrop-blur-[2px]"
    role="button"
    tabindex="0"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <!-- Modal Container -->
    <div
      transition:fly={{ y: 20, duration: 300, easing: cubicOut }}
      id="center-peek-modal"
      class="w-full md:max-w-4xl h-screen md:h-auto md:max-h-[90vh] md:min-h-[50vh] bg-[#0d0d0d] flex flex-col rounded-none md:rounded-xl shadow-2xl overflow-hidden font-sans border border-[var(--color-border-hairline)]"
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
        <div class="flex items-center gap-1 relative z-10">
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer disabled:opacity-30" disabled={!hasPrev} onclick={() => onNavigate('prev')} title="Previous Email">
            <ChevronUp class="w-4 h-4" />
          </button>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer disabled:opacity-30" disabled={!hasNext} onclick={() => onNavigate('next')} title="Next Email">
            <ChevronDown class="w-4 h-4" />
          </button>
        </div>

        <!-- Actions toolbar -->
        <!-- Actions toolbar (Desktop only) -->
        <div class="hidden md:flex items-center gap-1.5 relative z-10">
          {@render actionButtons()}
          <div class="w-px h-5 bg-[var(--color-border-hairline)] mx-1"></div>
          <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={onClose} title="Close peek (Esc)">
            <X class="w-4 h-4" />
          </button>
        </div>
        
        <!-- Close button (Mobile only) -->
        <div class="flex md:hidden items-center gap-1.5 relative z-10">
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

          <!-- Message HTML Render Content (Task 33: Body Sandboxing) -->
          <div class="border border-[var(--color-border-hairline)]/30 rounded-xl bg-[#131313]/10 overflow-hidden">
            <iframe 
              title="Email Body"
              sandbox="allow-same-origin allow-popups allow-popups-to-escape-sandbox"
              srcdoc={DOMPurify.sanitize(email.body, { WHOLE_DOCUMENT: true, ADD_TAGS: ['style'], ADD_ATTR: ['target'] })}
              class="w-full min-h-[20vh] bg-white"
              onload={(e) => { 
                const target = e.currentTarget as HTMLIFrameElement;
                if (target.contentWindow) {
                  // Reset height so scrollHeight shrinks if the new content is smaller
                  target.style.height = '0px';
                  target.style.height = (target.contentWindow.document.documentElement.scrollHeight + 20) + 'px';
                }
              }}
            ></iframe>
          </div>

          <!-- Attachments -->
          {#if email.attachments && email.attachments.length > 0}
            <div class="flex gap-3 overflow-x-auto pb-2">
              {#each email.attachments as attachment}
                <button 
                  onclick={() => {
                    import('@tauri-apps/plugin-http').then(http => {
                      import('@tauri-apps/plugin-fs').then(fs => {
                        import('@tauri-apps/api/path').then(path => {
                          console.log(`Downloading ${attachment.filename} via Tauri native HTTPS...`);
                        });
                      });
                    }).catch(() => {
                      window.open(`/api/v1/messages/${email.id}/attachments/${attachment.filename}`, '_blank');
                    });
                  }}
                  class="flex items-center gap-3 px-3 py-2 bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg shrink-0 cursor-pointer hover:bg-[var(--color-canvas-hover)] transition-colors text-left"
                >
                  <div class="w-8 h-8 rounded bg-purple-500/10 flex items-center justify-center text-purple-400">
                    <Paperclip class="w-4 h-4" />
                  </div>
                  <div class="flex flex-col">
                    <span class="text-xs font-semibold text-white">{attachment.filename}</span>
                    <span class="text-[10px] text-[var(--color-text-secondary)]">{(attachment.size / 1024 / 1024).toFixed(1)} MB</span>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
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

      <!-- Mobile Bottom Actions Bar -->
      <div class="md:hidden border-t border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)] px-4 py-2 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-1.5 w-full justify-between">
          {@render actionButtons()}
        </div>
      </div>
    </div>
  </div>
{/if}

<svelte:window onclick={() => activeMenu = null} />

{#snippet actionButtons()}
  <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => { onToggleUnread(email!.id); }} title={email!.isUnread ? "Mark as Read" : "Mark as Unread"}>
    {#if email!.isUnread}
      <MailOpen class="w-4 h-4 text-[var(--color-text-secondary)]" />
    {:else}
      <Mail class="w-4 h-4 text-[var(--color-text-secondary)]" />
    {/if}
  </button>
  
  <Dropdown 
    isOpen={activeMenu === 'snooze'}
    onClose={() => activeMenu = null}
  >
    {#snippet trigger()}
      <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'snooze' ? null : 'snooze'; }} title="Snooze">
        <Clock class="w-4 h-4 text-[var(--color-text-secondary)]" />
      </button>
    {/snippet}
    {#snippet content()}
      <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer" onclick={() => { onSnooze!(email!.id, 'later_today'); activeMenu = null; }}>Later Today</button>
      <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer" onclick={() => { onSnooze!(email!.id, 'tomorrow'); activeMenu = null; }}>Tomorrow</button>
      <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer" onclick={() => { onSnooze!(email!.id, 'next_week'); activeMenu = null; }}>Next Week</button>
    {/snippet}
  </Dropdown>

  <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={() => { onArchive(email!.id); }} title={email!.isArchived ? "Unarchive Email (e)" : "Archive Email (e)"}>
    {#if email!.isArchived}
      <ArchiveRestore class="w-4 h-4 text-[var(--color-text-secondary)]" />
    {:else}
      <Archive class="w-4 h-4 text-[var(--color-text-secondary)]" />
    {/if}
  </button>
  
  <button class="p-1.5 rounded-lg hover:bg-red-500/20 text-red-400 transition-colors cursor-pointer" onclick={() => { onDelete(email!.id); }} title="Delete Email">
    <Trash2 class="w-4 h-4" />
  </button>
  
  <button class="p-1.5 rounded-lg hover:bg-orange-500/20 text-orange-400 transition-colors cursor-pointer" onclick={() => { onReportSpam!(email!.id); }} title="Report Spam">
    <AlertOctagon class="w-4 h-4" />
  </button>

  <Dropdown 
    isOpen={activeMenu === 'move'}
    onClose={() => activeMenu = null}
  >
    {#snippet trigger()}
      <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'move' ? null : 'move'; }} title="Move To">
        <Folder class="w-4 h-4 text-[var(--color-text-secondary)]" />
      </button>
    {/snippet}
    {#snippet content()}
      <div class="px-2 py-1.5 border-b border-[var(--color-border-hairline)] sticky top-0 bg-[var(--color-canvas-modal)]">
        <input type="text" bind:value={moveSearchQuery} placeholder="Move to..." class="w-full bg-transparent border-none outline-none text-sm text-white px-2 py-1 placeholder-[var(--color-text-secondary)]" />
      </div>
      <div class="overflow-y-auto flex-1 py-1">
        {#each allLabels.filter((l: string) => l.toLowerCase().includes(moveSearchQuery.toLowerCase())) as label}
          <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer" onclick={() => { onMoveTo!(email!.id, label); activeMenu = null; }}>{label}</button>
        {/each}
      </div>
    {/snippet}
  </Dropdown>
  
  <Dropdown 
    isOpen={activeMenu === 'label'}
    onClose={() => activeMenu = null}
  >
    {#snippet trigger()}
      <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'label' ? null : 'label'; }} title="Labels">
        <Tag class="w-4 h-4 text-[var(--color-text-secondary)]" />
      </button>
    {/snippet}
    {#snippet content()}
      <div class="px-2 py-1.5 border-b border-[var(--color-border-hairline)] sticky top-0 bg-[var(--color-canvas-modal)]">
        <input type="text" bind:value={labelSearchQuery} placeholder="Search labels..." class="w-full bg-transparent border-none outline-none text-sm text-white px-2 py-1 placeholder-[var(--color-text-secondary)]" />
      </div>
      <div class="overflow-y-auto flex-1 py-1">
        {#each allLabels.filter((l: string) => l.toLowerCase().includes(labelSearchQuery.toLowerCase())) as label}
          <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer flex items-center justify-between" onclick={() => { onAddLabel!(email!.id, label); activeMenu = null; }}>
            <span>{label}</span>
            {#if email!.labels.includes(label)}<Check class="w-3.5 h-3.5 text-blue-400" />{/if}
          </button>
        {/each}
        {#if labelSearchQuery.trim() !== '' && !allLabels.some((l: string) => l.toLowerCase() === labelSearchQuery.toLowerCase())}
          <button class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-blue-400 transition-colors cursor-pointer" onclick={() => { onAddLabel!(email!.id, labelSearchQuery.trim()); activeMenu = null; }}>
            + Create "{labelSearchQuery.trim()}"
          </button>
        {/if}
      </div>
    {/snippet}
  </Dropdown>

  <div class="w-px h-5 bg-[var(--color-border-hairline)] mx-1 hidden md:block"></div>

  <Dropdown 
    isOpen={activeMenu === 'more'}
    onClose={() => activeMenu = null}
  >
    {#snippet trigger()}
      <button class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer" onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'more' ? null : 'more'; }} title="More Options">
        <MoreVertical class="w-4 h-4 text-[var(--color-text-secondary)]" />
      </button>
    {/snippet}
    {#snippet content()}
      <div class="py-1">
        {#each [
          { id: 'print', icon: FileText, label: 'Print', action: () => window.print() },
          { id: 'mute', icon: BellOff, label: 'Mute Conversation', action: () => onMute!(email!.id) },
          { id: 'phishing', icon: ShieldAlert, label: 'Report Phishing', action: () => onReportPhishing!(email!.id) },
          { id: 'block', icon: UserX, label: 'Block Sender', action: () => onBlockSender!(email!.senderEmail) },
          { id: 'event', icon: CalendarPlus, label: 'Create Event', action: () => onCreateEvent!(email!.id) },
          { id: 'filter', icon: Filter, label: 'Filter messages like these', action: () => onFilterMessages!(email!.senderEmail) },
          { id: 'download', icon: Download, label: 'Download message', action: () => onDownloadMessage!(email!.id) }
        ] as option}
          {@const OptionIcon = option.icon}
          <button class="w-full text-left flex items-center gap-2 px-4 py-2 text-sm hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-primary)] transition-colors cursor-pointer" onclick={() => { option.action(); activeMenu = null; }}>
            <OptionIcon class="w-4 h-4 text-[var(--color-text-secondary)]" />
            {option.label}
          </button>
        {/each}
      </div>
    {/snippet}
  </Dropdown>
{/snippet}
