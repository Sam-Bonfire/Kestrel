<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ThreadList from '$lib/components/ThreadList.svelte';
  import CenterPeek from '$lib/components/CenterPeek.svelte';
  import ComposeModal from '$lib/components/ComposeModal.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import MailSettingsModal from '$lib/components/MailSettingsModal.svelte';
  import { SettingsModal } from '@kestrel/shared';
  import { AppShell, ReauthBanner } from '@kestrel/shared/components';
  import { authState, initAuth, logout, addRevokedAccount } from '@kestrel/shared/stores';
  import { replayOfflineQueue, searchMessages } from '@kestrel/shared/api';
  import { registerNotificationCategories } from '$lib/notifications';
  import { onMount, untrack } from 'svelte';

  onMount(() => {
    // Initial offline queue replay
    replayOfflineQueue().catch(console.error);
    
    // Setup online listener
    const onOnline = () => replayOfflineQueue().catch(console.error);
    window.addEventListener('online', onOnline);

    // Register interactive notification category (Reply / Mark as Read / Archive)
    registerNotificationCategories().catch(console.error);

    import('@tauri-apps/plugin-notification').then(({ onAction }) => {
      onAction(async (event: any) => {
        const messageId = event.notification?.id;
        if (!messageId) return;
        try {
          if (event.actionId === 'mark_read') {
            const api = await import('@kestrel/shared/api');
            await api.markAsRead(messageId);
            // Optimistically update the list so the UI reflects the change immediately
            allEmails = allEmails.map(e => e.id === messageId ? { ...e, isUnread: false } : e);
          } else if (event.actionId === 'archive') {
            const api = await import('@kestrel/shared/api');
            await api.archiveMessage(messageId);
            allEmails = allEmails.map(e => e.id === messageId ? { ...e, isArchived: true } : e);
          }
        } catch (err) {
          console.error('Notification action failed:', err);
        }
      });
    }).catch(() => {
      // Ignore if not in Tauri
    });
    
    return () => {
      window.removeEventListener('online', onOnline);
    };
  });

  // ── Accounts ────────────────────────────────────────────────────
  const accounts = [
    { id: '1', name: 'Personal Gmail',  email: 'alex@gmail.com',   color: '#EA4335', provider: 'google' },
    { id: '2', name: 'Work Outlook',    email: 'alex@kestrel.dev', color: '#0078D4', provider: 'outlook' },
  ];

  // ── App state ───────────────────────────────────────────────────
  let currentView      = $state('inbox');
  let searchQuery      = $state('');
  let selectedThreadId = $state<string | null>(null);
  let isComposeOpen    = $state(false);
  let isCommandOpen    = $state(false);
  let isSettingsOpen   = $state(false);
  let isMailSettingsOpen = $state(false);
  let activeAccountId  = $state('all');
  let isMobileSidebarOpen = $state(false);
  let initialReplyMode = $state<'reply'|'reply_all'|'forward'|null>(null);

  // Custom labels created dynamically by user
  let customLabels = $state<string[]>([]);

  // ── Real emails from API ───────────────────────────────
  let allEmails = $state<any[]>([]);
  let searchResults = $state<any[] | null>(null);
  let isLoading = $state(true);

  $effect(() => {
    if (authState.isInitialized && !authState.isAuthenticated) {
      import('$app/navigation').then(({ goto }) => goto('/login'));
      return;
    }
    if (authState.isAuthenticated) {
      import('@tauri-apps/plugin-notification').then(({ onAction }) => {
        onAction((event: any) => {
          if (event.actionId === 'reply' && event.inputValue) {
            import('@kestrel/shared/api').then(api => {
              api.sendMessage({
                to: event.notification.body.split('\n')[0] || 'unknown',
                subject: 'Re: Message',
                body: event.inputValue,
              });
            });
          }
        }).catch(err => console.warn("Notification action listener error:", err));
      }).catch(() => {});

      import('@kestrel/shared/api').then(({ getMessages, createSyncStream }) => {
        // Initial fetch
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
          console.error('Failed to load messages:', err);
          if (err?.status === 401 || err?.message?.includes('401') || err?.message?.includes('Unauthorized')) {
            logout();
          }
          isLoading = false;
        });

        // Set up SSE sync listener (Task 34)
        const eventSource = createSyncStream(localStorage.getItem('kestrel_token') || undefined);
        eventSource.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.event_type === 'auth_revocation') {
              if (data.account_id && data.provider) {
                addRevokedAccount(data.account_id, data.provider);
              }
            } else if (data.type === 'new_mail' || data.type === 'sync_complete') {
              console.log('SSE Sync event received, refreshing messages...');
              // Just refetch the list for now
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
              });
              
              // Trigger Tauri native notification (Task 35)
              if (data.type === 'new_mail') {
                import('@tauri-apps/plugin-notification').then(({ sendNotification }) => {
                  sendNotification({
                    title: 'New Email Received',
                    body: data.subject || 'You have a new message',
                    actionTypeId: 'new_email_actions'
                  });
                }).catch(() => {
                  // Ignore if Tauri is not available (e.g. running in browser)
                });
              }
            }
          } catch (e) {
            console.error('Error parsing SSE event:', e);
          }
        };

        return () => {
          eventSource.close();
        };
      });
    }
  });

  // Derived labels list
  let allLabels = $derived(
    Array.from(new Set([...allEmails.flatMap(e => e.labels), ...customLabels]))
  );

  let unreadCount = $derived.by(() => {
    const total = allEmails.filter(e => e.isUnread && !e.isTrash && !e.isSpam).length;
    const current = activeAccountId === 'all' ? total : allEmails.filter(e => e.accountId === activeAccountId && e.isUnread && !e.isTrash && !e.isSpam).length;
    if (total === 0) return 0;
    return activeAccountId === 'all' ? `${total}` : `${current} / ${total}`;
  });
  let inboxCount = $derived.by(() => {
    const total = allEmails.filter(e => e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft).length;
    const current = activeAccountId === 'all' ? total : allEmails.filter(e => e.accountId === activeAccountId && e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft).length;
    if (total === 0) return 0;
    return activeAccountId === 'all' ? `${total}` : `${current} / ${total}`;
  });

  let viewCounts = $derived.by(() => {
    const getCountStr = (filterFn: (e: typeof allEmails[0]) => boolean) => {
      const total = allEmails.filter(filterFn).length;
      const current = activeAccountId === 'all' ? total : allEmails.filter(e => e.accountId === activeAccountId && filterFn(e)).length;
      if (total === 0) return 0;
      return activeAccountId === 'all' ? `${total}` : `${current} / ${total}`;
    };

    const counts: Record<string, string | number> = {};

    counts['inbox'] = getCountStr(e => e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft);
    counts['unread'] = getCountStr(e => e.isUnread && !e.isTrash && !e.isSpam);
    counts['sent'] = getCountStr(e => e.isUnread && e.labels.includes('Sent'));
    counts['drafts'] = getCountStr(e => e.isDraft);
    counts['starred'] = getCountStr(e => e.isUnread && e.isStarred && !e.isTrash);
    counts['github'] = getCountStr(e => e.isUnread && e.sender === 'GitHub' && !e.isTrash);
    counts['all-mail'] = getCountStr(e => e.isUnread && !e.isTrash);
    counts['spam'] = getCountStr(e => e.isUnread && e.isSpam);
    counts['trash'] = getCountStr(e => e.isUnread && e.isTrash);

    allLabels.forEach((lbl: string) => {
      counts[`label-${lbl}`] = getCountStr(e => !e.isTrash && e.isUnread && e.labels.some((l: string) => l.toLowerCase() === lbl.toLowerCase()));
    });

    return counts;
  });

  // ── Filtered thread list ─────────────────────────────────────────
  let threads = $derived(
    allEmails
      .filter(e => {
        if (activeAccountId !== 'all' && e.accountId !== activeAccountId) return false;
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
          return e.labels.some((l: string) => l.toLowerCase() === lbl.toLowerCase());
        }
        return true;
      })
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
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
        category: e.category,
        provider: accounts.find(a => a.id === e.accountId)?.provider || 'unknown',
        accountColor: accounts.find(a => a.id === e.accountId)?.color || '#6B7280'
      }))
  );

  let finalThreads = $derived(
    searchResults 
      ? searchResults.map(e => ({
          id: e.id,
          sender: e.sender_name || e.sender_email,
          senderEmail: e.sender_email,
          subject: e.subject || '(no subject)',
          snippet: e.snippet || '',
          date: formatDate(new Date(e.date_received * 1000).toISOString()),
          isUnread: !e.is_read,
          isStarred: false,
          hasAttachment: false,
          labels: [],
          category: 'Primary'
        }))
      : threads
  );

  // Debounced search logic
  let searchTimeout: any;
  $effect(() => {
    if (searchQuery.trim().length > 0) {
      clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => {
        searchMessages(searchQuery.trim()).then(res => {
          searchResults = res.results;
        }).catch(err => {
          console.error("Search error:", err);
          searchResults = null;
        });
      }, 300);
    } else {
      searchResults = null;
    }
  });

  let activeEmail = $derived.by(() => {
    if (!selectedThreadId) return null;
    let base = allEmails.find(e => e.id === selectedThreadId);
    if (!base) return null;
    if (fullBodies[selectedThreadId]) {
      return { ...base, body: fullBodies[selectedThreadId].body_html || fullBodies[selectedThreadId].body_text || base.body };
    }
    return base;
  });

  let fullBodies = $state<Record<string, any>>({});

  $effect(() => {
    if (selectedThreadId && authState.isAuthenticated) {
      // Mark as read automatically when opened
      const email = untrack(() => allEmails.find(e => e.id === selectedThreadId));
      if (email && email.isUnread) {
        import('@kestrel/shared/api').then(api => {
          api.markAsRead(selectedThreadId!).catch(err => console.error(err));
        });
        untrack(() => {
          allEmails = allEmails.map(e => e.id === selectedThreadId ? { ...e, isUnread: false } : e);
        });
      }

      if (!fullBodies[selectedThreadId]) {
        import('@kestrel/shared/api').then(({ getMessage }) => {
          getMessage(selectedThreadId!).then(res => {
            fullBodies[selectedThreadId!] = res;
          }).catch(err => console.error('Failed to load full message:', err));
        });
      }
    }
  });

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
  function advanceSelectionAndModify(id: string, modification: any) {
    const idx = threads.findIndex(t => t.id === id);
    let nextId = selectedThreadId;
    if (selectedThreadId === id && idx !== -1) {
      const nextIdx = idx < threads.length - 1 ? idx + 1 : idx - 1;
      nextId = nextIdx >= 0 ? threads[nextIdx].id : null;
    }
    allEmails = allEmails.map(e => e.id === id ? { ...e, ...modification } : e);
    selectedThreadId = nextId;
  }

  function archive(id: string) {
    const email = allEmails.find(e => e.id === id);
    if (!email) return;
    const newState = !email.isArchived;
    import('@kestrel/shared/api').then(({ archiveMessage }) => archiveMessage(id).catch(err => console.error(err)));
    advanceSelectionAndModify(id, { isArchived: newState });
  }
  function trash(id: string) {
    import('@kestrel/shared/api').then(({ trashMessage }) => trashMessage(id).catch(err => console.error(err)));
    advanceSelectionAndModify(id, { isTrash: true });
  }
  function snooze(id: string) {
    const ts = Math.floor(Date.now() / 1000) + 3600; // Snooze for 1 hour
    advanceSelectionAndModify(id, { isArchived: false, snoozed_until: ts });
    import('@kestrel/shared/api').then(api => { api.apiClient.post(`/messages/${id}/snooze`, { snoozed_until: ts }).catch(console.error); });
  }

  function reportSpam(id: string) {
    advanceSelectionAndModify(id, { isSpam: true, isTrash: true });
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
      labels: e.labels.map((l: string) => l === oldVal ? newVal : l)
    }));
    customLabels = customLabels.map((l: string) => l === oldVal ? newVal : l);
  }

  function deleteLabel(label: string) {
    allEmails = allEmails.map(e => ({
      ...e,
      labels: e.labels.filter((l: string) => l !== label)
    }));
    customLabels = customLabels.filter((l: string) => l !== label);
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

  function moveTo(threadId: string, folderOrLabel: string) {
    allEmails = allEmails.map(e => {
      if (e.id === threadId) {
        const newLabels = [folderOrLabel];
        import('@kestrel/shared/api').then(api => api.updateLabels(threadId, newLabels));
        return { ...e, labels: newLabels, isArchived: true };
      }
      return e;
    });
  }

  // Handlers for new email features
  function muteThread(id: string) {
    import('@kestrel/shared/api').then(api => api.muteMessage(id));
    applyLabel(id, 'Muted');
    archive(id);
  }
  function reportPhishing(id: string) {
    import('@kestrel/shared/api').then(api => api.reportPhishing(id));
    reportSpam(id);
  }
  function blockSender(emailAddress: string) {
    import('@kestrel/shared/api').then(api => api.blockSender(emailAddress));
  }
  function createEventFromEmail(id: string) {
    const parent = allEmails.find(e => e.id === id);
    if (!parent) return;
    const bodyText = (fullBodies[id] && (fullBodies[id].body_text || fullBodies[id].body_html)) || parent.body;
    
    const params = new URLSearchParams();
    params.set('title', parent.subject || 'New Event from Email');
    params.set('description', bodyText.substring(0, 500)); // Truncate if needed
    
    // In Tauri, use the deep link to launch the calendar.
    if ((window as any).__TAURI_INTERNALS__) {
      import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
        // We simulate a deep link internally or rely on OS handler
        // A simple way is to use window.location.href or a hidden anchor
        // Wait, window.open with a custom scheme works in most OS to trigger deep links.
        window.open(`kestrel://calendar/new?${params.toString()}`, '_self');
      }).catch(err => console.error(err));
    } else {
      const url = new URL(window.location.origin + '/calendar');
      url.search = params.toString();
      window.open(url.toString(), '_blank');
    }
  }
  function filterMessages(emailAddress: string) {
    // In a real app this would open the settings/filters modal prefilled with the sender
    searchQuery = `from:${emailAddress}`;
  }
  function downloadMessage(id: string) {
    import('@kestrel/shared/api').then(api => {
      const url = api.getEmlDownloadUrl(id);
      window.open(url, '_blank');
    });
  }

  function initiateReply(id: string) {
    initialReplyMode = 'reply';
    selectedThreadId = id;
  }
  function initiateReplyAll(id: string) {
    initialReplyMode = 'reply_all';
    selectedThreadId = id;
  }
  function initiateForward(id: string) {
    initialReplyMode = 'forward';
    selectedThreadId = id;
  }
  async function handleSendCompose(draft: { to: string; subject: string; body: string }) {
    isComposeOpen = false;
    const newMsg = {
      id: `sent-${Date.now()}`,
      accountId: activeAccountId,
      sender: 'Me',
      senderEmail: 'me@kestrel.dev',
      to: draft.to,
      subject: draft.subject || '(no subject)',
      body: draft.body,
      timestamp: new Date().toISOString(),
      isUnread: false,
      isStarred: false,
      isArchived: false,
      isTrash: false,
      isDraft: false,
      isSpam: false,
      hasAttachment: false,
      labels: ['Sent'],
      category: 'Primary'
    };
    allEmails = [newMsg, ...allEmails];
    import('@kestrel/shared/api').then(api => api.sendMessage(draft));
  }

  async function handleSendReply(id: string, text: string) {
    const parent = allEmails.find(e => e.id === id);
    if (!parent) return;
    const replyMsg = {
      id: `reply-${Date.now()}`,
      accountId: parent.accountId,
      sender: 'Me',
      senderEmail: 'me@kestrel.dev',
      to: parent.senderEmail,
      subject: `Re: ${parent.subject.replace(/^Re:\s*/i, '')}`,
      body: text,
      timestamp: new Date().toISOString(),
      isUnread: false,
      isStarred: false,
      isArchived: false,
      isTrash: false,
      isDraft: false,
      isSpam: false,
      hasAttachment: false,
      labels: ['Sent'],
      category: 'Primary'
    };
    allEmails = [replyMsg, ...allEmails];
    import('@kestrel/shared/api').then(api => api.sendMessage({
      to: parent.senderEmail,
      subject: replyMsg.subject,
      body: text,
      thread_id: id
    }));
  }

  function handleCommandSelect(cmd: string) {
    if (cmd === 'compose') isComposeOpen = true;
    else if (cmd === 'inbox') currentView = 'inbox';
    else if (cmd === 'settings') {
      isCommandOpen = false;
      isSettingsOpen = true;
    }
    else if (cmd.startsWith('view-')) currentView = cmd.replace('view-', '');
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
  import { isTyping } from '$lib/utils/keyboard';

  onMount(() => {
    initAuth();
    
    // Deep Link Listener for OAuth Callbacks
    if ((window as any).__TAURI_INTERNALS__) {
      import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
        onOpenUrl((urls) => {
          for (const url of urls) {
            if (url.startsWith('kestrel://oauth/callback')) {
              isSettingsOpen = true;
              // We'd want to focus the accounts tab if we had one here, but isSettingsOpen exposes the shared SettingsModal.
              // We can also trigger a re-fetch of accounts here.
              // The simplest way to signal the settings modal to load accounts is toggling it open.
            }
          }
        });
      }).catch(err => console.error("Failed to init deep-link plugin", err));
    }

    const handler = (e: KeyboardEvent) => {
      if (!authState.isAuthenticated) return;
      if (isTyping(e)) return;
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') { e.preventDefault(); isCommandOpen = true; }
      if (e.key === 'c') isComposeOpen = true;
      if (e.key === 'Escape') { selectedThreadId = null; isCommandOpen = false; isComposeOpen = false; }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });
</script>

<AppShell bind:isMobileSidebarOpen>
  <ReauthBanner />
  {#snippet sidebar()}
    <Sidebar
      {currentView}
      onSelectView={(v: any) => { currentView = v; selectedThreadId = null; isMobileSidebarOpen = false; }}
      onComposeClick={() => { isComposeOpen = true; isMobileSidebarOpen = false; }}
      onOpenMailSettings={() => { isMailSettingsOpen = true; isMobileSidebarOpen = false; }}
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
  {/snippet}

    <!-- Mail panel: full width thread list, no reader pane -->
    <ThreadList
      threads={finalThreads}
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
      onSnooze={snooze}
      onToggleUnread={toggleUnread}
      onBulkArchive={bulkArchive}
      onBulkDelete={bulkDelete}
      onBulkToggleUnread={bulkToggleUnread}
      onBulkToggleStar={bulkToggleStar}
      onApplyLabel={applyLabel}
      onMoveTo={moveTo}
      onReply={initiateReply}
      onReplyAll={initiateReplyAll}
      onForward={initiateForward}
      onMute={muteThread}
      onReportSpam={reportSpam}
      onOpenMobileSidebar={() => isMobileSidebarOpen = true}
    />
    
    <!-- Mobile FAB for Compose -->
    <button
      onclick={() => isComposeOpen = true}
      class="lg:hidden fixed bottom-20 right-6 w-14 h-14 bg-blue-500 rounded-full flex items-center justify-center shadow-[0_4px_20px_rgba(59,130,246,0.4)] text-white hover:bg-blue-600 transition-transform active:scale-95 z-30"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>
    </button>

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
      initialReplyMode={initialReplyMode}
      onClose={() => { selectedThreadId = null; initialReplyMode = null; }}
      onNavigate={navigatePeek}
      hasPrev={threads.findIndex(t => t.id === selectedThreadId) > 0}
      hasNext={threads.findIndex(t => t.id === selectedThreadId) < threads.length - 1}
      onArchive={archive}
      onDelete={trash}
      onSnooze={snooze}
      onToggleStar={toggleStar}
      onToggleUnread={toggleUnread}
      onAddLabel={applyLabel}
      onRemoveLabel={removeLabel}
      onMoveTo={moveTo}
      onSendReply={handleSendReply}
      allLabels={allLabels}
      onReportSpam={reportSpam}
      onMute={muteThread}
      onReportPhishing={reportPhishing}
      onBlockSender={blockSender}
      onCreateEvent={createEventFromEmail}
      onFilterMessages={filterMessages}
      onDownloadMessage={downloadMessage}
    />
  {/if}

  <ComposeModal
    isOpen={isComposeOpen}
    onClose={() => isComposeOpen = false}
    onSend={async (draft) => {
      try {
        const api = await import('@kestrel/shared/api');
        await api.sendMessage({
          to: draft.to,
          subject: draft.subject,
          body: draft.body,
        });
        isComposeOpen = false;
      } catch (err) {
        console.error('Failed to send message:', err);
        // Optionally show toast error here
      }
    }}
  />

  <CommandPalette 
    isOpen={isCommandOpen} 
    onClose={() => isCommandOpen = false} 
    onSelectCommand={handleCommandSelect}
  />
  <SettingsModal
    isOpen={isSettingsOpen}
    onClose={() => isSettingsOpen = false}
  />
  <MailSettingsModal
    isOpen={isMailSettingsOpen}
    onClose={() => isMailSettingsOpen = false}
  />
</AppShell>
