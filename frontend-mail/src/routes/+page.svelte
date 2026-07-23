<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ThreadList from '$lib/components/ThreadList.svelte';
  import CenterPeek from '$lib/components/CenterPeek.svelte';
  import ComposeModal from '$lib/components/ComposeModal.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import { Login } from '@kestrel/shared/components';
  import { authState } from '@kestrel/shared/stores';
  import { onMount } from 'svelte';

  // ── Accounts ────────────────────────────────────────────────────
  const accounts = [
    { id: '1', name: 'Personal Gmail',  email: 'alex@gmail.com',   color: '#EA4335' },
    { id: '2', name: 'Work Outlook',    email: 'alex@kestrel.dev', color: '#0078D4' },
  ];

  // ── App state ───────────────────────────────────────────────────
  let currentView      = $state('inbox');
  let searchQuery      = $state('');
  let selectedThreadId = $state<string | null>(null);
  let isComposeOpen    = $state(false);
  let isCommandOpen    = $state(false);
  let activeAccountId  = $state('1');
  let isMobileSidebarOpen = $state(false);

  // Custom labels created dynamically by user
  let customLabels = $state<string[]>([]);

  // ── Real emails from API ───────────────────────────────
  let allEmails = $state<any[]>([]);
  let isLoading = $state(true);

  $effect(() => {
    if (authState.isAuthenticated) {
      import('@kestrel/shared/api').then(({ getMessages }) => {
        getMessages().then(res => {
          allEmails = res.messages.map((m: any, index: number) => ({
            id: m.id,
            accountId: (m.sender_email?.includes('kestrel') || m.recipients?.includes('kestrel') || m.subject?.includes('Vercel') || index % 3 === 0) ? '2' : '1',
            sender: m.sender_name || m.sender_email,
            senderEmail: m.sender_email,
            to: 'me',
            subject: m.subject || '(no subject)',
            body: m.snippet || '', 
            timestamp: new Date(m.date_received * 1000).toISOString(),
            isUnread: !m.is_read,
            isStarred: (m.labels ? JSON.parse(m.labels) : []).includes('STARRED'),
            isArchived: m.is_archived,
            isTrash: false,
            isDraft: false,
            isSpam: false,
            hasAttachment: m.has_attachments,
            labels: m.labels ? JSON.parse(m.labels) : [],
            category: 'Primary'
          }));
          isLoading = false;
        }).catch(err => {
          console.error(err);
          isLoading = false;
        });
      });
    }
  });

  // Derived labels list
  let allLabels = $derived(
    Array.from(new Set([...allEmails.flatMap(e => e.labels), ...customLabels]))
  );

  let unreadCount = $derived(allEmails.filter(e => e.accountId === activeAccountId && e.isUnread && !e.isTrash && !e.isSpam).length);
  let inboxCount = $derived(allEmails.filter(e => e.accountId === activeAccountId && e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft).length);

  let viewCounts = $derived.by(() => {
    const accEmails = allEmails.filter(e => e.accountId === activeAccountId);
    const counts: Record<string, number> = {};

    counts['inbox'] = accEmails.filter(e => e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft).length;
    counts['unread'] = accEmails.filter(e => e.isUnread && !e.isTrash && !e.isSpam).length;
    counts['sent'] = accEmails.filter(e => e.isUnread && e.labels.includes('Sent')).length;
    counts['drafts'] = accEmails.filter(e => e.isDraft).length;
    counts['starred'] = accEmails.filter(e => e.isUnread && e.isStarred && !e.isTrash).length;
    counts['github'] = accEmails.filter(e => e.isUnread && e.sender === 'GitHub' && !e.isTrash).length;
    counts['all-mail'] = accEmails.filter(e => e.isUnread && !e.isTrash).length;
    counts['spam'] = accEmails.filter(e => e.isUnread && e.isSpam).length;
    counts['trash'] = accEmails.filter(e => e.isUnread && e.isTrash).length;

    allLabels.forEach(lbl => {
      const cnt = accEmails.filter(e => !e.isTrash && e.isUnread && e.labels.some(l => l.toLowerCase() === lbl.toLowerCase())).length;
      counts[`label-${lbl}`] = cnt;
    });

    return counts;
  });

  // ── Filtered thread list ─────────────────────────────────────────
  let threads = $derived(
    allEmails
      .filter(e => {
        if (e.accountId !== activeAccountId) return false;
        if (currentView === 'inbox')    return !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft;
        if (currentView === 'unread')   return e.isUnread && !e.isTrash && !e.isSpam;
        if (currentView === 'sent')     return e.labels.includes('Sent');
        if (currentView === 'drafts')   return e.isDraft;
        if (currentView === 'starred')  return e.isStarred && !e.isTrash;
        if (currentView === 'spam')     return e.isSpam;
        if (currentView === 'trash')    return e.isTrash;
        if (currentView === 'github')   return e.sender === 'GitHub' && !e.isTrash;
        if (currentView === 'all-mail') return !e.isTrash;
        if (currentView.startsWith('label-')) {
          const lbl = currentView.replace('label-', '');
          return e.labels.some(l => l.toLowerCase() === lbl.toLowerCase());
        }
        return true;
      })
      .filter(e => {
        if (!searchQuery.trim()) return true;
        const q = searchQuery.toLowerCase();
        return (
          e.sender.toLowerCase().includes(q) ||
          e.subject.toLowerCase().includes(q) ||
          e.body.toLowerCase().includes(q)
        );
      })
      .map(e => ({
        id: e.id,
        sender: e.sender,
        senderEmail: e.senderEmail,
        subject: e.subject,
        snippet: e.body.replace(/<[^>]*>?/gm, '').substring(0, 110) + '…',
        date: formatDate(e.timestamp),
        isUnread: e.isUnread,
        isStarred: e.isStarred,
        hasAttachment: false,
        labels: e.labels,
        category: e.category
      }))
  );

  let activeEmail = $derived(
    selectedThreadId ? allEmails.find(e => e.id === selectedThreadId) ?? null : null
  );

  function formatDate(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const sameDay = d.toDateString() === now.toDateString();
    return sameDay
      ? d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      : d.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }

  // ── Email actions ────────────────────────────────────────────────
  function toggleStar(id: string) {
    const email = allEmails.find(e => e.id === id);
    if (!email) return;
    const newState = !email.isStarred;
    allEmails = allEmails.map(e => e.id === id ? { ...e, isStarred: newState } : e);
    import('@kestrel/shared/api').then(api => api.toggleStar(id, newState));
  }
  function toggleUnread(id: string) {
    const email = allEmails.find(e => e.id === id);
    if (!email) return;
    const newState = !email.isUnread;
    allEmails = allEmails.map(e => e.id === id ? { ...e, isUnread: newState } : e);
    import('@kestrel/shared/api').then(api => {
        if (!newState) api.markAsRead(id);
        // Note: The backend doesn't have a markAsUnread endpoint yet, only markAsRead, but we can do it via bulk if needed or ignore for now.
    });
  }
  function archive(id: string) {
    import('@kestrel/shared/api').then(({ archiveMessage }) => archiveMessage(id));
    allEmails = allEmails.map(e => e.id === id ? { ...e, isArchived: true } : e);
    if (selectedThreadId === id) selectedThreadId = null;
  }
  function trash(id: string) {
    import('@kestrel/shared/api').then(({ trashMessage }) => trashMessage(id));
    allEmails = allEmails.map(e => e.id === id ? { ...e, isTrash: true } : e);
    if (selectedThreadId === id) selectedThreadId = null;
  }
  function navigatePeek(dir: 'prev' | 'next') {
    const idx = threads.findIndex(t => t.id === selectedThreadId);
    if (idx === -1) return;
    const next = dir === 'prev' ? idx - 1 : idx + 1;
    if (next >= 0 && next < threads.length) selectedThreadId = threads[next].id;
  }

  // Label Actions
  function renameLabel(oldVal: string, newVal: string) {
    allEmails = allEmails.map(e => ({
      ...e,
      labels: e.labels.map(l => l === oldVal ? newVal : l)
    }));
    customLabels = customLabels.map(l => l === oldVal ? newVal : l);
  }

  function deleteLabel(label: string) {
    allEmails = allEmails.map(e => ({
      ...e,
      labels: e.labels.filter(l => l !== label)
    }));
    customLabels = customLabels.filter(l => l !== label);
  }

  function createNewLabel(label: string) {
    customLabels = [...customLabels, label];
  }

  function applyLabel(threadId: string, label: string) {
    allEmails = allEmails.map(e => {
      if (e.id === threadId && !e.labels.includes(label)) {
        const newLabels = [...e.labels, label];
        import('@kestrel/shared/api').then(api => api.updateLabels(threadId, newLabels));
        return { ...e, labels: newLabels };
      }
      return e;
    });
  }

  function removeLabel(threadId: string, label: string) {
    allEmails = allEmails.map(e => {
      if (e.id === threadId) {
        const newLabels = e.labels.filter((l: string) => l !== label);
        import('@kestrel/shared/api').then(api => api.updateLabels(threadId, newLabels));
        return { ...e, labels: newLabels };
      }
      return e;
    });
  }

  // Bulk Actions
  function bulkArchive(ids: string[]) {
    allEmails = allEmails.map(e => ids.includes(e.id) ? { ...e, isArchived: true } : e);
    import('@kestrel/shared/api').then(api => api.bulkAction(ids, 'archive', true));
  }
  function bulkDelete(ids: string[]) {
    allEmails = allEmails.map(e => ids.includes(e.id) ? { ...e, isTrash: true } : e);
    import('@kestrel/shared/api').then(api => api.bulkAction(ids, 'trash', true));
  }
  function bulkToggleUnread(ids: string[], isUnread: boolean) {
    allEmails = allEmails.map(e => ids.includes(e.id) ? { ...e, isUnread } : e);
    import('@kestrel/shared/api').then(api => api.bulkAction(ids, 'mark_read', !isUnread));
  }
  function bulkToggleStar(ids: string[], isStarred: boolean) {
    allEmails = allEmails.map(e => ids.includes(e.id) ? { ...e, isStarred } : e);
    import('@kestrel/shared/api').then(api => api.bulkAction(ids, 'toggle_star', isStarred));
  }

  // ── Keyboard shortcuts ───────────────────────────────────────────
  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      if (!authState.isAuthenticated) return;
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') { e.preventDefault(); isCommandOpen = true; }
      if (e.key === 'c') isComposeOpen = true;
      if (e.key === 'Escape') { selectedThreadId = null; isCommandOpen = false; isComposeOpen = false; }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });
</script>

{#if !authState.isAuthenticated}
  <Login />
{:else}
<div class="flex h-screen w-screen overflow-hidden bg-[var(--color-canvas-base)] relative">

  <!-- Mobile Drawer Overlay -->
  {#if isMobileSidebarOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 bg-black/60 z-40 lg:hidden backdrop-blur-sm" onclick={() => isMobileSidebarOpen = false}></div>
  {/if}

  <!-- Sidebar Container -->
  <div class="fixed inset-y-0 left-0 z-50 transform transition-transform duration-300 lg:relative lg:translate-x-0 {isMobileSidebarOpen ? 'translate-x-0' : '-translate-x-full'} shadow-2xl lg:shadow-none">
    <Sidebar
      {currentView}
      onSelectView={(v) => { currentView = v; selectedThreadId = null; isMobileSidebarOpen = false; }}
      onComposeClick={() => { isComposeOpen = true; isMobileSidebarOpen = false; }}
      bind:searchQuery
      {accounts}
      bind:activeAccountId
      {allLabels}
      onRenameLabel={renameLabel}
      onDeleteLabel={deleteLabel}
      onCreateNewLabel={createNewLabel}
      {inboxCount}
      {unreadCount}
      {viewCounts}
    />
  </div>

  <!-- Main Content Area -->
  <div class="flex-1 flex flex-col min-w-0 h-full relative">
    <!-- Mail panel: full width thread list, no reader pane -->
    <ThreadList
      {threads}
      {currentView}
      {selectedThreadId}
      {allLabels}
      onSelectThread={(id) => {
        selectedThreadId = id;
        import('@kestrel/shared/api').then(api => {
          api.markAsRead(id);
          api.getMessage(id).then(detail => {
            allEmails = allEmails.map(e => e.id === id ? {
              ...e, 
              isUnread: false,
              body: detail.body_html || detail.body_text || detail.snippet || '',
              to: detail.recipients || e.to
            } : e);
          });
        });
      }}
      onToggleStar={toggleStar}
      onArchive={archive}
      onDelete={trash}
      onToggleUnread={toggleUnread}
      onBulkArchive={bulkArchive}
      onBulkDelete={bulkDelete}
      onBulkToggleUnread={bulkToggleUnread}
      onBulkToggleStar={bulkToggleStar}
      onApplyLabel={applyLabel}
      onOpenMobileSidebar={() => isMobileSidebarOpen = true}
    />
    
    <!-- Mobile FAB for Compose -->
    <button
      onclick={() => isComposeOpen = true}
      class="lg:hidden fixed bottom-20 right-6 w-14 h-14 bg-blue-500 rounded-full flex items-center justify-center shadow-[0_4px_20px_rgba(59,130,246,0.4)] text-white hover:bg-blue-600 transition-transform active:scale-95 z-30"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>
    </button>
  </div>

  <!-- Mobile Bottom Navigation Bar -->
  <div class="lg:hidden fixed bottom-0 left-0 right-0 h-16 bg-[#131313] border-t border-white/5 flex items-center justify-around z-40 px-2 pb-safe">
    <button class="flex flex-col items-center justify-center w-16 h-full text-blue-400">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>
      <span class="text-[10px] mt-1 font-medium">Mail</span>
    </button>
    <button class="flex flex-col items-center justify-center w-16 h-full text-[var(--color-text-secondary)]">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
      <span class="text-[10px] mt-1 font-medium">Chat</span>
    </button>
    <button class="flex flex-col items-center justify-center w-16 h-full text-[var(--color-text-secondary)]">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
      <span class="text-[10px] mt-1 font-medium">Meet</span>
    </button>
  </div>

  <!-- Center peek modal overlay -->
  {#if activeEmail}
    <CenterPeek
      email={activeEmail}
      onClose={() => selectedThreadId = null}
      onNavigate={navigatePeek}
      hasPrev={threads.findIndex(t => t.id === selectedThreadId) > 0}
      hasNext={threads.findIndex(t => t.id === selectedThreadId) < threads.length - 1}
      onArchive={archive}
      onDelete={trash}
      onToggleStar={toggleStar}
      onToggleUnread={toggleUnread}
      onAddLabel={applyLabel}
      onRemoveLabel={removeLabel}
      onSendReply={(id, text) => alert(`Reply sent: "${text}"`)}
    />
  {/if}

  <ComposeModal
    isOpen={isComposeOpen}
    onClose={() => isComposeOpen = false}
    onSend={() => isComposeOpen = false}
  />

  <CommandPalette
    isOpen={isCommandOpen}
    onClose={() => isCommandOpen = false}
  />
</div>
{/if}
