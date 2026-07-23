<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    Star, Paperclip, Archive, Trash2, MailOpen, Mail, RotateCw, 
    ListFilter, Inbox, CheckSquare, Square, ChevronDown, Check,
    Clock, AlertTriangle, Sparkles, Tag, Plus, X, Folder, ChevronRight
  } from 'lucide-svelte';
  import { WindowControls } from '@kestrel/shared/components';
  import { 
    mailDenseMode, 
    labelCustomizations, 
    getLabelStyle 
  } from '@kestrel/shared';

  export interface EmailThread {
    id: string;
    sender: string;
    senderEmail: string;
    subject: string;
    snippet: string;
    date: string;
    isUnread: boolean;
    isStarred: boolean;
    hasAttachment: boolean;
    labels: string[];
    category?: string;
  }

  let {
    threads = [] as EmailThread[],
    selectedThreadId = $bindable<string | null>(null),
    currentView = 'inbox',
    onSelectThread = (id: string) => {},
    onToggleStar = (id: string) => {},
    onArchive = (id: string) => {},
    onDelete = (id: string) => {},
    onToggleUnread = (id: string) => {},
    onBulkArchive = (ids: string[]) => {},
    onBulkDelete = (ids: string[]) => {},
    onBulkToggleUnread = (ids: string[], isUnread: boolean) => {},
    onBulkToggleStar = (ids: string[], isStarred: boolean) => {},
    onApplyLabel = (id: string, label: string) => {},
    allLabels = [] as string[],
    onOpenMobileSidebar = () => {}
  } = $props<{
    threads?: EmailThread[];
    selectedThreadId?: string | null;
    currentView?: string;
    onSelectThread?: (id: string) => void;
    onToggleStar?: (id: string) => void;
    onArchive?: (id: string) => void;
    onDelete?: (id: string) => void;
    onToggleUnread?: (id: string) => void;
    onBulkArchive?: (ids: string[]) => void;
    onBulkDelete?: (ids: string[]) => void;
    onBulkToggleUnread?: (ids: string[], isUnread: boolean) => void;
    onBulkToggleStar?: (ids: string[], isStarred: boolean) => void;
    onApplyLabel?: (id: string, label: string) => void;
    allLabels?: string[];
    onOpenMobileSidebar?: () => void;
  }>();

  let selectedIndex = $state(0);
  let hoveredId = $state<string | null>(null);

  // Checkbox state tracking
  let checkedThreads = $state<Record<string, boolean>>({});

  // Context Menu state
  let threadContextMenu = $state<{ x: number; y: number; threadId: string } | null>(null);
  let showLabelDropdown = $state(false);

  // Filter Toolbar Toggle
  let showFiltersBar = $state(false);

  // Filtering states
  let activeCategory = $state<'All' | 'Primary' | 'Updates' | 'Social' | 'Forums'>('All');
  let activeLabelFilter = $state<'All' | string>('All');
  let unreadFilterOnly = $state(false);
  let hasAttachmentFilterOnly = $state(false);
  let activeDateRange = $state<'All' | 'Today' | 'This Week' | 'This Month'>('All');
  let showCategoryFilterDropdown = $state(false);
  let showLabelFilterDropdown = $state(false);
  let showDateRangeDropdown = $state(false);

  const viewLabels: Record<string, string> = {
    inbox: 'Inbox', unread: 'Unread', sent: 'Sent', drafts: 'Drafts',
    spam: 'Spam', trash: 'Trash', github: 'GitHub', 'all-mail': 'All Mail', starred: 'Starred'
  };

  let displayTitle = $derived(viewLabels[currentView] ?? currentView.replace('label-', ''));

  // Filtered threads displayed in list
  let filteredList = $derived(
    threads
      .filter((t: EmailThread) => {
        if (activeCategory === 'All') return true;
        return t.category === activeCategory;
      })
      .filter((t: EmailThread) => {
        if (activeLabelFilter === 'All') return true;
        return t.labels.includes(activeLabelFilter);
      })
      .filter((t: EmailThread) => {
        if (unreadFilterOnly && !t.isUnread) return false;
        return true;
      })
      .filter((t: EmailThread) => {
        if (hasAttachmentFilterOnly && !t.hasAttachment) return false;
        return true;
      })
      .filter((t: EmailThread) => {
        if (activeDateRange === 'All') return true;
        // Mock date filtering logic
        const today = new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
        if (activeDateRange === 'Today') return t.date === today || t.date.includes(':'); // Times usually mean today
        return true; // Simple stub for last 7 days
      })
  );

  let unreadCount = $derived(filteredList.filter((t: EmailThread) => t.isUnread).length);

  // Selection helpers
  let isAllChecked = $derived(
    filteredList.length > 0 && filteredList.every((t: EmailThread) => checkedThreads[t.id])
  );

  let anyChecked = $derived(
    filteredList.some((t: EmailThread) => checkedThreads[t.id])
  );

  let checkedIds = $derived(
    Object.keys(checkedThreads).filter(id => checkedThreads[id])
  );

  function toggleSelectAll() {
    if (isAllChecked) {
      checkedThreads = {};
    } else {
      const nextChecked: Record<string, boolean> = {};
      filteredList.forEach((t: EmailThread) => {
        nextChecked[t.id] = true;
      });
      checkedThreads = nextChecked;
    }
  }

  function toggleCheck(id: string, e: MouseEvent) {
    e.stopPropagation();
    checkedThreads = {
      ...checkedThreads,
      [id]: !checkedThreads[id]
    };
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (filteredList.length === 0) return;
    const tag = (event.target as HTMLElement).tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;

    if (event.key === 'j' || event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredList.length - 1);
      onSelectThread(filteredList[selectedIndex].id);
    } else if (event.key === 'k' || event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      onSelectThread(filteredList[selectedIndex].id);
    }
  }

  function handleThreadContextMenu(threadId: string, e: MouseEvent) {
    e.preventDefault();
    showLabelDropdown = false;
    threadContextMenu = {
      x: e.clientX,
      y: e.clientY,
      threadId
    };
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
    const handleGlobalClick = () => {
      if (threadContextMenu) threadContextMenu = null;
      showCategoryFilterDropdown = false;
      showLabelFilterDropdown = false;
      showDateRangeDropdown = false;
    };
    window.addEventListener('click', handleGlobalClick);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('click', handleGlobalClick);
    };
  });
</script>

<div class="flex-1 h-screen bg-[var(--color-canvas-base)] flex flex-col overflow-hidden font-sans pb-16 lg:pb-0">
  
  <!-- Thread list header -->
  <div 
    class="px-4 lg:pl-6 lg:pr-36 py-2 flex items-center justify-between shrink-0 bg-[var(--color-canvas-base)] cursor-default select-none relative"
  >
    <!-- Transparent drag handle that stops before WindowControls -->
    <div class="absolute inset-y-0 left-0 right-36" data-tauri-drag-region></div>
    <div class="flex items-center gap-2.5">
      <button 
        onclick={onOpenMobileSidebar}
        class="lg:hidden p-1.5 -ml-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] transition-colors active:scale-95"
        aria-label="Open Menu"
        title="Open Menu"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
      </button>
      <Inbox class="w-4 h-4 text-[var(--color-text-primary)] hidden sm:block" />
      <h1 class="text-sm font-semibold text-[var(--color-text-primary)] select-none">{displayTitle}</h1>
      {#if unreadCount > 0}
        <span class="text-[10px] font-mono text-[var(--color-text-secondary)] bg-[var(--color-canvas-hover)] px-1.5 py-0.5 rounded-md border border-[var(--color-border-hairline)]">
          {unreadCount} unread
        </span>
      {/if}
    </div>
    
    <div class="flex items-center gap-1.5 text-[var(--color-text-secondary)]">
      <button 
        onclick={() => toggleSelectAll()}
        class="p-1.5 rounded-lg hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors"
        title="Select all"
      >
        {#if isAllChecked}
          <CheckSquare class="w-3.5 h-3.5 text-blue-400" />
        {:else}
          <Square class="w-3.5 h-3.5" />
        {/if}
      </button>
      
      <!-- Filter Toggle Button in Toolbar -->
      <button 
        onclick={() => showFiltersBar = !showFiltersBar}
        class="p-1.5 rounded-lg hover:bg-[var(--color-canvas-hover)] hover:text-white transition-colors {showFiltersBar ? 'bg-[var(--color-canvas-hover)] text-white' : ''}" 
        title="Filter applied views"
      >
        <ListFilter class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>

  <!-- Collapsible Filter Toolbar Chips -->
  {#if showFiltersBar}
    <div class="px-6 py-2.5 bg-[var(--color-canvas-card)]/30 border-b border-[var(--color-border-hairline)]/50 flex flex-wrap items-center gap-2 text-xs text-[var(--color-text-primary)] animate-fadeIn">
      <!-- Category filter custom dropdown -->
      <div class="relative flex items-center gap-1.5 bg-[var(--color-canvas-card)] px-2.5 py-1 rounded-lg border border-[var(--color-border-hairline)] select-none">
        <span class="text-[var(--color-text-secondary)] font-mono text-[10px]">Category:</span>
        <button
          onclick={(e) => { e.stopPropagation(); showCategoryFilterDropdown = !showCategoryFilterDropdown; showLabelFilterDropdown = false; }}
          class="flex items-center gap-1 text-xs text-white cursor-pointer select-none bg-transparent border-none outline-none"
        >
          <span>{activeCategory === 'All' ? 'All Categories' : activeCategory}</span>
          <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
        </button>

        {#if showCategoryFilterDropdown}
          <div class="absolute left-0 top-full mt-1.5 w-44 bg-[#1a1919] border border-white/10 rounded-xl shadow-2xl z-50 py-1 font-sans text-xs">
            {#each ['All', 'Primary', 'Updates', 'Social', 'Forums'] as cat}
              <button
                onclick={() => { activeCategory = cat as any; showCategoryFilterDropdown = false; }}
                class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] transition-colors text-white cursor-pointer border-none bg-transparent"
              >
                <span>{cat === 'All' ? 'All Categories' : cat}</span>
                {#if activeCategory === cat}
                  <Check class="w-3 h-3 text-blue-400" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Label filter custom dropdown -->
      <div class="relative flex items-center gap-1.5 bg-[var(--color-canvas-card)] px-2.5 py-1 rounded-lg border border-[var(--color-border-hairline)] select-none">
        <span class="text-[var(--color-text-secondary)] font-mono text-[10px]">Label:</span>
        <button
          onclick={(e) => { e.stopPropagation(); showLabelFilterDropdown = !showLabelFilterDropdown; showCategoryFilterDropdown = false; }}
          class="flex items-center gap-1 text-xs text-white cursor-pointer select-none bg-transparent border-none outline-none"
        >
          <span>{activeLabelFilter === 'All' ? 'All Labels' : activeLabelFilter.split('/').pop()}</span>
          <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
        </button>

        {#if showLabelFilterDropdown}
          <div class="absolute left-0 top-full mt-1.5 w-48 bg-[#1a1919] border border-white/10 rounded-xl shadow-2xl z-50 py-1 font-sans text-xs max-h-48 overflow-y-auto">
            <button
              onclick={() => { activeLabelFilter = 'All'; showLabelFilterDropdown = false; }}
              class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] transition-colors text-white cursor-pointer border-none bg-transparent"
            >
              <span>All Labels</span>
              {#if activeLabelFilter === 'All'}
                <Check class="w-3 h-3 text-blue-400" />
              {/if}
            </button>
            {#each allLabels as label}
              <button
                onclick={() => { activeLabelFilter = label; showLabelFilterDropdown = false; }}
                class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] transition-colors text-white cursor-pointer border-none bg-transparent"
              >
                <span>{label.split('/').pop()}</span>
                {#if activeLabelFilter === label}
                  <Check class="w-3 h-3 text-blue-400" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Unread toggle pill -->
      <button
        onclick={() => unreadFilterOnly = !unreadFilterOnly}
        class="px-2.5 py-1 rounded-full text-xs border transition-colors cursor-pointer
          {unreadFilterOnly ? 'bg-blue-500/10 border-blue-500/30 text-blue-400 font-semibold' : 'bg-[var(--color-canvas-card)] border-[var(--color-border-hairline)] text-[var(--color-text-secondary)]'}"
      >
        Is unread
      </button>

      <!-- Date Range Filter -->
      <div class="relative flex items-center gap-1.5 bg-[var(--color-canvas-card)] px-2.5 py-1 rounded-lg border border-[var(--color-border-hairline)] select-none">
        <span class="text-[var(--color-text-secondary)] font-mono text-[10px]">Date:</span>
        <button
          onclick={(e) => { e.stopPropagation(); showDateRangeDropdown = !showDateRangeDropdown; showCategoryFilterDropdown = false; showLabelFilterDropdown = false; }}
          class="flex items-center gap-1 text-xs text-white cursor-pointer select-none bg-transparent border-none outline-none"
        >
          <span>{activeDateRange}</span>
          <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
        </button>

        {#if showDateRangeDropdown}
          <div class="absolute left-0 top-full mt-1.5 w-32 bg-[#1a1919] border border-white/10 rounded-xl shadow-2xl z-50 py-1 font-sans text-xs">
            {#each (['All', 'Today', 'This Week', 'This Month'] as const) as range}
              <button
                onclick={() => { activeDateRange = range; showDateRangeDropdown = false; }}
                class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] transition-colors text-white cursor-pointer border-none bg-transparent"
              >
                <span>{range}</span>
                {#if activeDateRange === range}
                  <Check class="w-3 h-3 text-blue-400" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Has Attachments toggle pill -->
      <button
        onclick={() => hasAttachmentFilterOnly = !hasAttachmentFilterOnly}
        class="px-2.5 py-1 rounded-full text-xs border transition-colors cursor-pointer
          {hasAttachmentFilterOnly ? 'bg-blue-500/10 border-blue-500/30 text-blue-400 font-semibold' : 'bg-[var(--color-canvas-card)] border-[var(--color-border-hairline)] text-[var(--color-text-secondary)]'}"
      >
        Has Attachments
      </button>

      <!-- Clear action -->
      {#if activeCategory !== 'All' || activeLabelFilter !== 'All' || unreadFilterOnly || hasAttachmentFilterOnly || activeDateRange !== 'All'}
        <button 
          onclick={() => { activeCategory = 'All'; activeLabelFilter = 'All'; unreadFilterOnly = false; hasAttachmentFilterOnly = false; activeDateRange = 'All'; }}
          class="text-[10px] text-white/50 hover:text-white underline cursor-pointer ml-auto"
        >
          Clear filters
        </button>
      {/if}
    </div>
  {/if}

  <!-- Bulk Action floating toolbar overlay -->
  {#if anyChecked}
    <div class="absolute bottom-6 left-1/2 -translate-x-1/2 z-50">
      <div class="px-4 py-2.5 bg-[#18181b] border border-white/10 rounded-2xl flex items-center justify-between shadow-2xl shrink-0 font-sans text-xs text-white min-w-[320px] ring-1 ring-white/5">
        <div class="flex items-center gap-2">
          <span class="w-5 h-5 flex items-center justify-center rounded-full bg-blue-500/20 text-blue-400 font-bold">{checkedIds.length}</span>
          <span class="text-[var(--color-text-secondary)] font-medium">selected</span>
        </div>
        <div class="flex items-center gap-1.5 ml-6">
          <button
            onclick={() => { onBulkToggleUnread(checkedIds, false); checkedThreads = {}; }}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
          >
            <MailOpen class="w-3.5 h-3.5" strokeWidth={1.5} />
          </button>
          <button
            onclick={() => { onBulkToggleStar(checkedIds, true); checkedThreads = {}; }}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
          >
            <Star class="w-3.5 h-3.5 text-amber-400" strokeWidth={1.5} />
          </button>
          <button
            onclick={() => { onBulkArchive(checkedIds); checkedThreads = {}; }}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
          >
            <Archive class="w-3.5 h-3.5 text-blue-400" strokeWidth={1.5} />
          </button>
          <div class="w-px h-4 bg-white/10 mx-1"></div>
          <button
            onclick={() => { onBulkDelete(checkedIds); checkedThreads = {}; }}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg hover:bg-red-500/20 text-red-400 transition-colors cursor-pointer"
          >
            <Trash2 class="w-3.5 h-3.5" strokeWidth={1.5} />
          </button>
          <div class="w-px h-4 bg-white/10 mx-1"></div>
          <button
            onclick={() => checkedThreads = {}}
            class="p-1.5 rounded-lg hover:bg-white/10 text-[var(--color-text-secondary)] transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Single-Line Row Thread items list -->
  <div class="flex-1 overflow-y-auto divide-y divide-[var(--color-border-hairline)]/10 px-2 py-1 bg-[var(--color-canvas-base)] relative">
    {#if filteredList.length === 0}
      <div class="flex flex-col items-center justify-center h-full gap-4 text-[var(--color-text-secondary)] animate-fadeIn">
        <div class="w-16 h-16 rounded-full bg-[var(--color-canvas-card)] flex items-center justify-center shadow-inner">
          <Inbox class="w-8 h-8 opacity-40 text-white" />
        </div>
        <div class="flex flex-col items-center gap-1">
          <span class="text-sm font-semibold text-white">No messages here</span>
          <span class="text-[11px] opacity-60">You're all caught up!</span>
        </div>
      </div>
    {:else}
      {#each filteredList as thread, i}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="group relative flex flex-col sm:flex-row sm:items-center bg-[var(--color-canvas-base)] hover:bg-[var(--color-canvas-hover)]/40 rounded-lg cursor-pointer transition-colors border border-transparent focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--color-canvas-base)] focus:outline-none
            {selectedThreadId === thread.id ? 'bg-[var(--color-canvas-hover)]/60 border-white/5' : ''}
            {$mailDenseMode ? 'py-2 sm:py-1 px-3 sm:px-3 min-h-[50px] sm:min-h-[32px]' : 'py-3 sm:py-2.5 px-4 min-h-[64px] sm:min-h-[44px]'}"
          onclick={() => { selectedIndex = i; onSelectThread(thread.id); }}
          oncontextmenu={(e) => handleThreadContextMenu(thread.id, e)}
          onmouseenter={() => hoveredId = thread.id}
          onmouseleave={() => hoveredId = null}
          role="button"
          tabindex="0"
        >
          <div class="flex items-center w-full sm:w-auto min-w-0">
            <!-- Left Checkbox (Hidden on mobile) -->
            <div class="mr-3 shrink-0 hidden sm:flex items-center opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity {checkedThreads[thread.id] ? '!opacity-100' : ''}">
              <button 
                onclick={(e) => toggleCheck(thread.id, e)}
                class="w-4 h-4 rounded border flex items-center justify-center transition-colors bg-[#1c1b1b]/40 cursor-pointer
                  {checkedThreads[thread.id] ? 'border-blue-500 bg-blue-500/10' : 'border-white/10 hover:border-white/30'}"
              >
                {#if checkedThreads[thread.id]}
                  <Check class="w-2.5 h-2.5 text-blue-500" />
                {/if}
              </button>
            </div>

            <!-- Unread status blue dot -->
            <div class="w-3 shrink-0 mr-1 flex items-center justify-center">
              {#if thread.isUnread}
                <span class="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]"></span>
              {/if}
            </div>

            <!-- Sender column (Mobile: flex with date) -->
            <div class="w-full sm:w-36 md:w-44 sm:pr-4 shrink-0 flex justify-between sm:justify-start items-center text-xs truncate">
              <span class="{thread.isUnread ? 'font-bold text-white' : 'font-medium text-neutral-200'} truncate">{thread.sender}</span>
              <span class="font-mono text-[10px] tracking-tight text-[var(--color-text-secondary)] sm:hidden shrink-0">{thread.date}</span>
            </div>
          </div>

          <!-- Subject + Snippet (Stacked on mobile, inline on desktop) -->
          <div class="flex-1 sm:pr-6 flex flex-col sm:flex-row sm:items-baseline gap-0.5 sm:gap-2 overflow-hidden text-xs min-w-0 pl-4 sm:pl-0 mt-0.5 sm:mt-0">
            <span class="text-white truncate shrink-0 sm:max-w-[150px] md:max-w-[250px] lg:max-w-md {thread.isUnread ? 'font-bold text-white' : 'font-semibold text-neutral-200'}">
              {thread.subject}
            </span>
            <span class="text-[var(--color-text-secondary)] truncate font-light text-[11px] flex-1 min-w-0 hidden sm:block">
              — {thread.snippet}
            </span>
            <span class="text-[var(--color-text-secondary)] truncate font-light text-[11px] flex-1 min-w-0 sm:hidden">
              {thread.snippet}
            </span>
          </div>

          <!-- Right labels + Date / hover buttons (Hidden date on mobile, moved to sender col) -->
          <div class="ml-auto shrink-0 flex items-center justify-end gap-3 relative min-w-[60px] pl-4 sm:pl-0">
            <!-- Static view: Labels + Attachment + Date -->
            <div class="flex items-center gap-3 text-xs text-[var(--color-text-secondary)] group-hover:opacity-0 transition-opacity duration-100">
              {#if thread.labels.length > 0}
                <div class="hidden md:flex items-center gap-1">
                  {#each thread.labels as label}
                    {@const labelStyle = getLabelStyle(label, $labelCustomizations)}
                    <span class="text-[9px] font-mono px-2 py-0.5 rounded-full border {labelStyle.bgColor} {labelStyle.textColor} {labelStyle.borderColor}">
                      {label.split('/').pop()}
                    </span>
                  {/each}
                </div>
              {/if}

              {#if thread.hasAttachment}
                <Paperclip class="w-3.5 h-3.5 text-[var(--color-text-secondary)]/50 shrink-0" strokeWidth={1.5} />
              {/if}
              
              <span class="font-mono text-[11px] tracking-tight shrink-0 w-12 text-right hidden sm:block">{thread.date}</span>
            </div>

            <!-- Floating action menu on hover (Matches screenshot: Star, Archive, Delete, CheckSquare, Clock, Tag) -->
            <div class="
              absolute right-0 top-1/2 -translate-y-1/2 hidden group-hover:flex items-center gap-0.5
              bg-[#1c1b1b] border border-white/10 rounded-lg p-0.5 shadow-2xl z-10
            ">
              <!-- Star -->
              <button
                onclick={(e) => { e.stopPropagation(); onToggleStar(thread.id); }}
                title="Star"
                class="p-1 rounded hover:bg-white/5 transition-colors cursor-pointer {thread.isStarred ? 'text-amber-400' : 'text-neutral-400 hover:text-white'}"
              >
                <Star class="w-3.5 h-3.5 {thread.isStarred ? 'fill-current' : ''}" />
              </button>

              <!-- Archive -->
              <button
                onclick={(e) => { e.stopPropagation(); onArchive(thread.id); }}
                title="Archive"
                class="p-1 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                <Archive class="w-3.5 h-3.5" />
              </button>

              <!-- Delete -->
              <button
                onclick={(e) => { e.stopPropagation(); onDelete(thread.id); }}
                title="Delete"
                class="p-1 rounded hover:bg-red-500/10 text-neutral-400 hover:text-red-400 transition-colors cursor-pointer"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>

              <!-- CheckSquare (Toggle read) -->
              <button
                onclick={(e) => { e.stopPropagation(); onToggleUnread(thread.id); }}
                title={thread.isUnread ? "Mark as Read" : "Mark as Unread"}
                class="p-1 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                <CheckSquare class="w-3.5 h-3.5" />
              </button>

              <!-- Clock (Snooze) -->
              <button
                onclick={(e) => { e.stopPropagation(); alert("Snoozed thread until tomorrow."); }}
                title="Snooze"
                class="p-1 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                <Clock class="w-3.5 h-3.5" />
              </button>

              <!-- Tag (Label) -->
              <button
                onclick={(e) => { e.stopPropagation(); handleThreadContextMenu(thread.id, e); }}
                title="Apply Label"
                class="p-1 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                <Tag class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- Right Click Context Menu Overlay -->
{#if threadContextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed bg-[#1a1919] border border-white/10 rounded-xl shadow-xl w-44 py-1 z-50 font-sans text-xs text-[var(--color-text-primary)]"
    style="left: {threadContextMenu.x}px; top: {threadContextMenu.y}px;"
    onclick={(e) => e.stopPropagation()}
    role="menu"
    tabindex="-1"
  >
    <button 
      onclick={() => { onSelectThread(threadContextMenu!.threadId); threadContextMenu = null; }}
      class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors"
    >
      <Inbox class="w-3.5 h-3.5 text-blue-400" />
      <span>Open Thread</span>
    </button>
    <button 
      onclick={() => { onToggleUnread(threadContextMenu!.threadId); threadContextMenu = null; }}
      class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors"
    >
      <Mail class="w-3.5 h-3.5 text-emerald-400" />
      <span>Mark Read/Unread</span>
    </button>
    <button 
      onclick={() => { onToggleStar(threadContextMenu!.threadId); threadContextMenu = null; }}
      class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors"
    >
      <Star class="w-3.5 h-3.5 text-amber-400" />
      <span>Star Thread</span>
    </button>
    <button 
      onclick={() => { onArchive(threadContextMenu!.threadId); threadContextMenu = null; }}
      class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors"
    >
      <Archive class="w-3.5 h-3.5 text-violet-400" />
      <span>Archive</span>
    </button>

    <!-- Labels sub-trigger -->
    <div class="relative border-t border-white/5 mt-1">
      <button 
        onclick={() => showLabelDropdown = !showLabelDropdown}
        class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center justify-between cursor-pointer transition-colors"
      >
        <div class="flex items-center gap-2">
          <Tag class="w-3.5 h-3.5 text-pink-400" />
          <span>Apply Label</span>
        </div>
        <ChevronRight class="w-3 h-3 text-[var(--color-text-secondary)] transition-transform {showLabelDropdown ? 'rotate-90' : ''}" />
      </button>

      {#if showLabelDropdown}
        <div class="absolute left-full top-0 ml-1 bg-[#1a1919] border border-white/10 rounded-xl shadow-xl w-44 py-1 max-h-40 overflow-y-auto">
          {#each allLabels as label}
            <button
              onclick={() => {
                onApplyLabel(threadContextMenu!.threadId, label);
                threadContextMenu = null;
                showLabelDropdown = false;
              }}
              class="w-full px-3 py-1.5 text-left text-[11px] hover:bg-canvas-hover transition-colors truncate"
            >
              {label.split('/').pop()}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <button 
      onclick={() => { onDelete(threadContextMenu!.threadId); threadContextMenu = null; }}
      class="w-full px-3 py-2 text-left hover:bg-red-500/10 text-red-400 flex items-center gap-2 cursor-pointer transition-colors border-t border-white/5 mt-1"
    >
      <Trash2 class="w-3.5 h-3.5" />
      <span>Delete Thread</span>
    </button>
  </div>
{/if}
