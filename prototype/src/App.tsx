import React, { useState, useEffect, useMemo } from 'react';
import { Sparkles, Info, X } from 'lucide-react';
import Sidebar from './components/Sidebar';
import MailList from './components/MailList';
import CenterPeek from './components/CenterPeek';
import ComposeModal from './components/ComposeModal';
import CalendarView from './components/CalendarView';
import DesignSystem from './components/DesignSystem';
import { mockEmails } from './data/mockEmails';
import { Email, ViewType, FilterState } from './types';

export default function App() {
  // Path state for simple routing support
  const [path, setPath] = useState(() => window.location.pathname);

  useEffect(() => {
    const handlePopState = () => {
      setPath(window.location.pathname);
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const handleNavigate = (newPath: string) => {
    window.history.pushState(null, '', newPath);
    setPath(newPath);
  };

  // Load emails from local storage or default to mock data
  const [emails, setEmails] = useState<Email[]>(() => {
    const saved = localStorage.getItem('kestrel_emails_v2');
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error('Failed to parse saved emails', e);
      }
    }
    return mockEmails;
  });

  // Navigation and Filter States
  const [currentView, setView] = useState<ViewType>(() => {
    try {
      return (localStorage.getItem('kestrel_mail_default_view') as ViewType) || 'inbox';
    } catch {
      return 'inbox';
    }
  });
  const [selectedLabel, setSelectedLabel] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  // Mail settings states
  const [mailDenseMode, setMailDenseMode] = useState<boolean>(() => {
    try {
      return localStorage.getItem('kestrel_mail_dense_mode') === 'true';
    } catch {
      return false;
    }
  });

  const [mailDefaultLandingView, setMailDefaultLandingView] = useState<string>(() => {
    try {
      return localStorage.getItem('kestrel_mail_default_view') || 'inbox';
    } catch {
      return 'inbox';
    }
  });

  const [mailSignature, setMailSignature] = useState<string>(() => {
    try {
      return localStorage.getItem('kestrel_mail_signature') || 'Sent from Kestrel Mail';
    } catch {
      return 'Sent from Kestrel Mail';
    }
  });
  
  // Custom Filters (Category [Gmail], Custom removable chips, etc.)
  const [filterState, setFilterState] = useState<FilterState>({
    category: null,
    label: null,
    isUnread: false,
    showArchived: false,
    customChips: [
      { id: 'from-github-not', field: 'From', value: 'github.com' } // Preloaded matching screenshot
    ]
  });

  // Modal display states
  const [selectedEmail, setSelectedEmail] = useState<Email | null>(null);
  const [isComposeOpen, setIsComposeOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  // Non-intrusive Toast state
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'info' | 'error' } | null>(null);

  // Custom label customizations state initialized from localStorage
  const [labelCustomizations, setLabelCustomizations] = useState<Record<string, { iconName: string; colorName: string }>>(() => {
    try {
      const stored = localStorage.getItem('label_customizations');
      return stored ? JSON.parse(stored) : {};
    } catch {
      return {};
    }
  });

  const handleRenameLabel = (oldName: string, newName: string) => {
    if (!oldName || !newName || oldName === newName) return;

    setEmails(prevEmails => {
      return prevEmails.map(email => {
        if (email.labels && email.labels.includes(oldName)) {
          return {
            ...email,
            labels: email.labels.map(l => l === oldName ? newName : l)
          };
        }
        return email;
      });
    });

    setLabelCustomizations(prev => {
      const updated = { ...prev };
      if (updated[oldName]) {
        updated[newName] = updated[oldName];
        delete updated[oldName];
      }
      localStorage.setItem('label_customizations', JSON.stringify(updated));
      return updated;
    });

    showToast(`Renamed label "${oldName}" to "${newName}"`, 'success');
  };

  const handleUpdateLabelCustomization = (label: string, iconName: string, colorName: string) => {
    setLabelCustomizations(prev => {
      const updated = {
        ...prev,
        [label]: { iconName, colorName }
      };
      localStorage.setItem('label_customizations', JSON.stringify(updated));
      return updated;
    });
    showToast(`Updated style for label "${label}"`, 'success');
  };

  const handleDeleteLabel = (label: string) => {
    setEmails(prevEmails => {
      return prevEmails.map(email => {
        if (email.labels && email.labels.includes(label)) {
          return {
            ...email,
            labels: email.labels.filter(l => l !== label)
          };
        }
        return email;
      });
    });

    setLabelCustomizations(prev => {
      const updated = { ...prev };
      delete updated[label];
      localStorage.setItem('label_customizations', JSON.stringify(updated));
      return updated;
    });

    showToast(`Deleted label "${label}"`, 'success');
  };

  // Sync state with LocalStorage
  useEffect(() => {
    localStorage.setItem('kestrel_emails_v2', JSON.stringify(emails));
  }, [emails]);

  // Show a message toast
  const showToast = (message: string, type: 'success' | 'info' | 'error' = 'success') => {
    setToast({ message, type });
  };

  // Automatically dismiss toast after 4 seconds
  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => {
        setToast(null);
      }, 4000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  // Keyboard shortcut listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Close active modals on Esc
      if (e.key === 'Escape') {
        setSelectedEmail(null);
        setIsComposeOpen(false);
      }
      // Compose on 'c' key if not typing in inputs/textareas
      if (e.key === 'c' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
        e.preventDefault();
        setIsComposeOpen(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Filter & Navigate within the currently active list
  const currentFilteredList = useMemo(() => {
    return emails.filter(email => {
      // Apply the same filter logic used in MailList to determine previous/next navigation
      if (currentView === 'inbox') {
        if (email.isArchived || email.isTrash || email.isSpam || email.isDraft) return false;
      } else if (currentView === 'unread') {
        if (!email.isUnread || email.isTrash || email.isSpam) return false;
      } else if (currentView === 'sent') {
        if (!email.isDraft && (email.labels.includes('Sent') || email.senderEmail === 'user@workspace.io')) return true;
        return false;
      } else if (currentView === 'drafts') {
        if (!email.isDraft) return false;
      } else if (currentView === 'spam') {
        if (!email.isSpam) return false;
      } else if (currentView === 'trash') {
        if (!email.isTrash) return false;
      } else if (currentView === 'starred') {
        if (!email.isStarred || email.isTrash) return false;
      } else if (currentView === 'github') {
        if (email.sender !== 'GitHub' || email.isTrash) return false;
      } else if (currentView === 'label' && selectedLabel) {
        const matchesLabel = email.labels.some(l => l === selectedLabel || l.startsWith(selectedLabel + '/'));
        if (!matchesLabel || email.isTrash) return false;
      } else if (currentView === 'categories' && selectedCategory) {
        if (email.category !== selectedCategory || email.isTrash) return false;
      }

      if (filterState.category && email.category !== filterState.category) return false;
      if (filterState.label && !email.labels.some(l => l === filterState.label || l.startsWith(filterState.label + '/'))) return false;
      if (filterState.isUnread && !email.isUnread) return false;
      
      if (currentView !== 'all-mail' && currentView !== 'trash' && currentView !== 'spam') {
        if (!filterState.showArchived && email.isArchived) return false;
      }

      const githubChipActive = filterState.customChips.some(c => c.id === 'from-github-not');
      if (githubChipActive && email.sender === 'GitHub') return false;

      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        return (
          email.sender.toLowerCase().includes(query) ||
          email.senderEmail.toLowerCase().includes(query) ||
          email.subject.toLowerCase().includes(query) ||
          email.body.toLowerCase().includes(query) ||
          email.labels.some(l => l.toLowerCase().includes(query))
        );
      }

      return true;
    });
  }, [emails, currentView, selectedLabel, selectedCategory, filterState, searchQuery]);

  // Handle email click row -> open center peek
  const handleEmailClick = (email: Email) => {
    setSelectedEmail(email);
    // Mark as read immediately on click
    if (email.isUnread) {
      setEmails(prev => prev.map(e => e.id === email.id ? { ...e, isUnread: false } : e));
    }
  };

  // Navigation within the Center Peek modal (Previous/Next)
  const handlePeekNavigation = (direction: 'prev' | 'next') => {
    if (!selectedEmail) return;
    const currentIndex = currentFilteredList.findIndex(e => e.id === selectedEmail.id);
    if (currentIndex === -1) return;

    let targetIndex = direction === 'prev' ? currentIndex - 1 : currentIndex + 1;
    if (targetIndex >= 0 && targetIndex < currentFilteredList.length) {
      const nextEmail = currentFilteredList[targetIndex];
      setSelectedEmail(nextEmail);
      if (nextEmail.isUnread) {
        setEmails(prev => prev.map(e => e.id === nextEmail.id ? { ...e, isUnread: false } : e));
      }
    }
  };

  const hasPrev = selectedEmail ? currentFilteredList.findIndex(e => e.id === selectedEmail.id) > 0 : false;
  const hasNext = selectedEmail ? currentFilteredList.findIndex(e => e.id === selectedEmail.id) < currentFilteredList.length - 1 : false;

  // Actions
  const handleArchive = (id: string) => {
    setEmails(prev => prev.map(e => e.id === id ? { ...e, isArchived: true } : e));
    showToast("Email successfully archived.", "success");
  };

  const handleDelete = (id: string) => {
    setEmails(prev => prev.map(e => e.id === id ? { ...e, isTrash: true } : e));
    showToast("Email moved to Trash.", "info");
  };

  const handleToggleStar = (id: string) => {
    setEmails(prev => prev.map(e => e.id === id ? { ...e, isStarred: !e.isStarred } : e));
  };

  const handleToggleUnread = (id: string) => {
    setEmails(prev => prev.map(e => e.id === id ? { ...e, isUnread: !e.isUnread } : e));
    showToast("Marked as unread.", "info");
  };

  const handleAddLabel = (id: string, label: string) => {
    setEmails(prev => prev.map(e => {
      if (e.id === id) {
        if (!e.labels.includes(label)) {
          return { ...e, labels: [...e.labels, label] };
        }
      }
      return e;
    }));
  };

  const handleRemoveLabel = (id: string, label: string) => {
    setEmails(prev => prev.map(e => {
      if (e.id === id) {
        return { ...e, labels: e.labels.filter(l => l !== label) };
      }
      return e;
    }));
  };

  // Bulk Actions
  const handleBulkArchive = (ids: string[]) => {
    setEmails(prev => prev.map(e => ids.includes(e.id) ? { ...e, isArchived: true } : e));
    showToast(`${ids.length} emails successfully archived.`, "success");
  };

  const handleBulkDelete = (ids: string[]) => {
    setEmails(prev => prev.map(e => ids.includes(e.id) ? { ...e, isTrash: true } : e));
    showToast(`${ids.length} emails moved to Trash.`, "info");
  };

  const handleBulkToggleUnread = (ids: string[], isUnread: boolean) => {
    setEmails(prev => prev.map(e => ids.includes(e.id) ? { ...e, isUnread } : e));
    showToast(`Marked ${ids.length} emails as ${isUnread ? 'unread' : 'read'}.`, "info");
  };

  const handleBulkToggleStar = (ids: string[], isStarred: boolean) => {
    setEmails(prev => prev.map(e => ids.includes(e.id) ? { ...e, isStarred } : e));
    showToast(`Updated star status for ${ids.length} emails.`, "info");
  };

  // Send interactive outbound email
  const handleSendEmail = (newEmailData: Omit<Email, 'id' | 'timestamp'>) => {
    const newId = `sent-${Date.now()}`;
    const newEmail: Email = {
      ...newEmailData,
      id: newId,
      timestamp: new Date().toISOString()
    };
    setEmails(prev => [newEmail, ...prev]);
    showToast(`Outbound email successfully sent to ${newEmail.to}.`, "success");
  };

  // Reply inline handler
  const handleSendReply = (emailId: string, replyBody: string) => {
    const parentEmail = emails.find(e => e.id === emailId);
    if (!parentEmail) return;

    const replyId = `reply-${Date.now()}`;
    const newReply: Email = {
      id: replyId,
      sender: 'Developer',
      senderEmail: 'user@workspace.io',
      to: parentEmail.senderEmail,
      subject: `Re: ${parentEmail.subject}`,
      body: `<p style="font-family: sans-serif; line-height: 1.6; color: #e5e2e1;">${replyBody.replace(/\n/g, '<br/>')}</p>`,
      timestamp: new Date().toISOString(),
      isUnread: false,
      isArchived: false,
      isStarred: false,
      isDraft: false,
      isSpam: false,
      isTrash: false,
      hasAttachment: false,
      category: 'Primary',
      labels: ['Sent', 'Replied']
    };

    setEmails(prev => [newReply, ...prev]);
    showToast(`Reply successfully delivered to ${parentEmail.senderEmail}.`, "success");
  };

  // Smart Heuristics-based Auto Label
  const handleAutoLabelAll = () => {
    let labeledCount = 0;
    setEmails(prev => prev.map(e => {
      let updatedLabels = [...e.labels];
      let updated = false;

      // Smart rules based on sender names & keywords
      if (e.sender === 'Michael Page India' && !updatedLabels.includes('Careers') && !updatedLabels.includes('Job Alerts')) {
        updatedLabels.push('Careers', 'Job Alerts');
        updated = true;
      }
      if (e.sender === 'Indeed' && !updatedLabels.includes('Careers') && !updatedLabels.includes('Indeed Matches')) {
        updatedLabels.push('Careers', 'Indeed Matches');
        updated = true;
      }
      if (e.sender === 'cc.statements@axis.bank.in' && !updatedLabels.includes('Finance') && !updatedLabels.includes('Statements')) {
        updatedLabels.push('Finance', 'Statements');
        updated = true;
      }
      if (e.sender === 'Google Play' && !updatedLabels.includes('Subscriptions') && !updatedLabels.includes('Billing')) {
        updatedLabels.push('Subscriptions', 'Billing');
        updated = true;
      }
      if (e.sender === 'Cisco' && !updatedLabels.includes('Recruitment') && !updatedLabels.includes('Careers')) {
        updatedLabels.push('Recruitment', 'Careers');
        updated = true;
      }
      if (e.sender === 'Amex Careers' && !updatedLabels.includes('American Express') && !updatedLabels.includes('Referrals')) {
        updatedLabels.push('American Express', 'Referrals');
        updated = true;
      }
      if (e.sender === 'GitHub' && !updatedLabels.includes('GitHub') && !updatedLabels.includes('DevOps')) {
        updatedLabels.push('DevOps');
        updated = true;
      }
      if (e.sender === 'Notion' && !updatedLabels.includes('Notion Mail') && !updatedLabels.includes('Updates [Gmail]')) {
        updatedLabels.push('Notion Mail');
        updated = true;
      }

      if (updated) {
        labeledCount++;
        return { ...e, labels: Array.from(new Set(updatedLabels)) };
      }
      return e;
    }));

    showToast(`AI Auto-labelled ${labeledCount} matching threads with structural tags.`, "success");
  };

  // Re-seed original mock values for convenience
  const handleRefreshInbox = () => {
    setEmails(mockEmails);
    setView('inbox');
    setSelectedLabel(null);
    setSelectedCategory(null);
    setFilterState({
      category: null,
      label: null,
      isUnread: false,
      showArchived: false,
      customChips: [
        { id: 'from-github-not', field: 'From', value: 'github.com' }
      ]
    });
    showToast("Inbox refreshed to baseline demo state.", "info");
  };

  // Update dynamic state calculations if modal is open to capture real-time changes
  const activePeekEmail = selectedEmail ? emails.find(e => e.id === selectedEmail.id) || selectedEmail : null;

  return (
    <div id="notion-mail-root" className="w-screen h-screen flex bg-canvas-base text-text-primary font-sans overflow-hidden antialiased relative">
      
      {/* Main Column focused panel based on active path route */}
      {path === '/calendar' ? (
        <CalendarView
          onBackToMail={() => handleNavigate('/')}
          showToast={showToast}
        />
      ) : (path === '/design-system' || path === '/style-guide' || path === '/styleguide' || path === '/components') ? (
        <DesignSystem onBack={() => handleNavigate('/')} />
      ) : (
        <>
          {/* Mobile Sidebar Backdrop */}
          {isSidebarOpen && (
            <div 
              id="sidebar-mobile-backdrop" 
              className="fixed inset-0 bg-black/45 backdrop-blur-[1px] z-30 md:hidden"
              onClick={() => setIsSidebarOpen(false)}
            />
          )}

          {/* Sidebar Panel */}
          <Sidebar
            emails={emails}
            currentView={currentView}
            setView={setView}
            selectedLabel={selectedLabel}
            setSelectedLabel={setSelectedLabel}
            selectedCategory={selectedCategory}
            setSelectedCategory={setSelectedCategory}
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
            onComposeClick={() => setIsComposeOpen(true)}
            isMobileOpen={isSidebarOpen}
            onCloseMobile={() => setIsSidebarOpen(false)}
            labelCustomizations={labelCustomizations}
            onRenameLabel={handleRenameLabel}
            onUpdateLabelCustomization={handleUpdateLabelCustomization}
            onDeleteLabel={handleDeleteLabel}
            currentPath={path}
            onNavigate={handleNavigate}
            mailDenseMode={mailDenseMode}
            setMailDenseMode={setMailDenseMode}
            mailDefaultLandingView={mailDefaultLandingView}
            setMailDefaultLandingView={setMailDefaultLandingView}
            mailSignature={mailSignature}
            setMailSignature={setMailSignature}
          />

          <MailList
            emails={emails}
            currentView={currentView}
            selectedLabel={selectedLabel}
            selectedCategory={selectedCategory}
            filterState={filterState}
            setFilterState={setFilterState}
            onEmailClick={handleEmailClick}
            onRefresh={handleRefreshInbox}
            searchQuery={searchQuery}
            onToggleStar={handleToggleStar}
            onArchive={handleArchive}
            onDelete={handleDelete}
            onToggleUnread={handleToggleUnread}
            onAddLabel={handleAddLabel}
            showToast={showToast}
            onToggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
            onBulkArchive={handleBulkArchive}
            onBulkDelete={handleBulkDelete}
            onBulkToggleUnread={handleBulkToggleUnread}
            onBulkToggleStar={handleBulkToggleStar}
            labelCustomizations={labelCustomizations}
            mailDenseMode={mailDenseMode}
          />
        </>
      )}

      {/* Center Peek Modal Overlay */}
      {selectedEmail && (
        <CenterPeek
          email={activePeekEmail}
          onClose={() => setSelectedEmail(null)}
          onNavigate={handlePeekNavigation}
          hasPrev={hasPrev}
          hasNext={hasNext}
          onArchive={handleArchive}
          onDelete={handleDelete}
          onToggleStar={handleToggleStar}
          onToggleUnread={handleToggleUnread}
          onAddLabel={handleAddLabel}
          onRemoveLabel={handleRemoveLabel}
          onSendReply={handleSendReply}
          labelCustomizations={labelCustomizations}
        />
      )}

      {/* Compose Email Panel Overlay */}
      {isComposeOpen && (
        <ComposeModal
          onClose={() => setIsComposeOpen(false)}
          onSend={handleSendEmail}
        />
      )}

      {/* Floating Status Toast notification */}
      {toast && (
        <div 
          id="toast-notification-popup"
          className="fixed bottom-6 right-6 z-50 flex items-center gap-2.5 px-4 py-3 bg-[#1c1b1b] border border-white/10 rounded-md shadow-2xl animate-fade-in"
        >
          <Sparkles className="w-4 h-4 text-white" />
          <span className="text-xs font-mono text-text-primary font-medium">{toast.message}</span>
          <button 
            id="btn-toast-dismiss"
            onClick={() => setToast(null)}
            className="p-0.5 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer ml-2"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

    </div>
  );
}
