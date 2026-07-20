import React, { useState, useMemo, useRef } from 'react';
import { 
  Inbox, 
  Paperclip, 
  Plus, 
  X, 
  RotateCw, 
  Sparkles, 
  ChevronDown, 
  Check, 
  MailWarning, 
  CheckSquare, 
  Archive, 
  Trash2, 
  Star,
  Maximize2,
  ListFilter,
  Clock,
  Tag,
  Menu,
  Mail,
  MailOpen
} from 'lucide-react';
import { Email, FilterState, ViewType } from '../types';
import { getLabelConfig, iconMapping, colorConfigs } from '../data/labelConfig';

interface MailListProps {
  emails: Email[];
  currentView: ViewType;
  selectedLabel: string | null;
  selectedCategory: string | null;
  filterState: FilterState;
  setFilterState: React.Dispatch<React.SetStateAction<FilterState>>;
  onEmailClick: (email: Email) => void;
  onRefresh: () => void;
  searchQuery: string;
  onToggleStar?: (id: string) => void;
  onArchive?: (id: string) => void;
  onDelete?: (id: string) => void;
  onToggleUnread?: (id: string) => void;
  onAddLabel?: (id: string, label: string) => void;
  showToast?: (message: string, type?: 'success' | 'info' | 'error') => void;
  onToggleSidebar?: () => void;
  onBulkArchive?: (ids: string[]) => void;
  onBulkDelete?: (ids: string[]) => void;
  onBulkToggleUnread?: (ids: string[], isUnread: boolean) => void;
  onBulkToggleStar?: (ids: string[], isStarred: boolean) => void;
  labelCustomizations: Record<string, { iconName: string; colorName: string }>;
  mailDenseMode?: boolean;
}

export default function MailList({
  emails,
  currentView,
  selectedLabel,
  selectedCategory,
  filterState,
  setFilterState,
  onEmailClick,
  onRefresh,
  searchQuery,
  onToggleStar,
  onArchive,
  onDelete,
  onToggleUnread,
  onAddLabel,
  showToast,
  onToggleSidebar,
  onBulkArchive,
  onBulkDelete,
  onBulkToggleUnread,
  onBulkToggleStar,
  labelCustomizations,
  mailDenseMode = false
}: MailListProps) {
  const [catDropdownOpen, setCatDropdownOpen] = useState(false);
  const [labelDropdownOpen, setLabelDropdownOpen] = useState(false);
  const [mobileFiltersOpen, setMobileFiltersOpen] = useState(false);
  const [checkedEmails, setCheckedEmails] = useState<Record<string, boolean>>({});

  // Context menu for individual email items
  const [emailContextMenu, setEmailContextMenu] = useState<{
    x: number;
    y: number;
    emailId: string;
  } | null>(null);

  // Dynamic label style resolver with customizations
  const getLabelStyle = (labelName: string) => {
    const base = getLabelConfig(labelName);
    const custom = labelCustomizations?.[labelName];
    if (custom) {
      const icon = iconMapping[custom.iconName] || base.icon;
      const colorStyle = colorConfigs[custom.colorName];
      if (colorStyle) {
        return {
          name: labelName,
          icon,
          color: custom.colorName,
          bgColor: colorStyle.bg,
          textColor: colorStyle.text,
          borderColor: colorStyle.border
        };
      }
    }
    return base;
  };

  const isSelectionMode = useMemo(() => {
    return Object.values(checkedEmails).some(Boolean);
  }, [checkedEmails]);

  // Touch handlers for long-press on mobile devices
  const touchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const touchStartPosRef = useRef<{ x: number; y: number } | null>(null);
  const justLongPressedRef = useRef<boolean>(false);

  const handleTouchStart = (e: React.TouchEvent, emailId: string) => {
    const touch = e.touches[0];
    touchStartPosRef.current = { x: touch.clientX, y: touch.clientY };
    
    if (touchTimerRef.current) clearTimeout(touchTimerRef.current);
    
    touchTimerRef.current = setTimeout(() => {
      justLongPressedRef.current = true;
      setCheckedEmails(prev => ({
        ...prev,
        [emailId]: !prev[emailId]
      }));
      if (showToast) {
        showToast("Selection mode activated.", "info");
      }
    }, 600); // 600ms long press duration
  };

  const handleTouchEnd = (e: React.TouchEvent) => {
    if (touchTimerRef.current) {
      clearTimeout(touchTimerRef.current);
      touchTimerRef.current = null;
    }
    if (justLongPressedRef.current) {
      e.preventDefault();
      setTimeout(() => {
        justLongPressedRef.current = false;
      }, 100);
    }
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!touchStartPosRef.current) return;
    const touch = e.touches[0];
    const dx = Math.abs(touch.clientX - touchStartPosRef.current.x);
    const dy = Math.abs(touch.clientY - touchStartPosRef.current.y);
    if (dx > 10 || dy > 10) {
      if (touchTimerRef.current) {
        clearTimeout(touchTimerRef.current);
        touchTimerRef.current = null;
      }
    }
  };

  const toggleCheck = (e: React.MouseEvent, emailId: string) => {
    e.stopPropagation();
    setCheckedEmails(prev => ({
      ...prev,
      [emailId]: !prev[emailId]
    }));
  };

  // Available unique labels and categories for filters
  const allCategories = ['All', 'Primary', 'Updates', 'Social', 'Promotions', 'Forums'];
  const allLabels = useMemo(() => {
    const labels = new Set<string>();
    emails.forEach(e => e.labels.forEach(l => labels.add(l)));
    return ['All', ...Array.from(labels)];
  }, [emails]);

  // Handle setting Category filter
  const selectCategoryFilter = (cat: string) => {
    setFilterState(prev => ({
      ...prev,
      category: cat === 'All' ? null : cat
    }));
    setCatDropdownOpen(false);
  };

  // Handle setting Label filter
  const selectLabelFilter = (lbl: string) => {
    setFilterState(prev => ({
      ...prev,
      label: lbl === 'All' ? null : lbl
    }));
    setLabelDropdownOpen(false);
  };

  // Remove a custom chip (e.g. From: Not "github.com")
  const removeCustomChip = (chipId: string) => {
    setFilterState(prev => ({
      ...prev,
      customChips: prev.customChips.filter(c => c.id !== chipId)
    }));
  };

  // Reset all filters
  const resetFilters = () => {
    setFilterState({
      category: null,
      label: null,
      isUnread: false,
      showArchived: false,
      customChips: []
    });
  };

  // Filter & Search Logic
  const filteredEmails = useMemo(() => {
    return emails.filter(email => {
      // 1. Folder structure views
      if (currentView === 'inbox') {
        if (email.isArchived || email.isTrash || email.isSpam || email.isDraft) return false;
      } else if (currentView === 'unread') {
        if (!email.isUnread || email.isTrash || email.isSpam) return false;
      } else if (currentView === 'sent') {
        // We simulate some sent emails
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
        if (!email.labels.includes(selectedLabel) || email.isTrash) return false;
      } else if (currentView === 'categories' && selectedCategory) {
        if (email.category !== selectedCategory || email.isTrash) return false;
      }

      // 2. Dynamic Dropdown Filters
      if (filterState.category && email.category !== filterState.category) return false;
      if (filterState.label && !email.labels.includes(filterState.label)) return false;
      if (filterState.isUnread && !email.isUnread) return false;
      
      // Default archive hiding (unless explicitly viewing All Mail, Trash, Spam or 'Show Archived' is enabled)
      if (currentView !== 'all-mail' && currentView !== 'trash' && currentView !== 'spam') {
        if (!filterState.showArchived && email.isArchived) return false;
      }

      // 3. Custom preloaded chips (e.g., From: Not "github.com")
      const githubChipActive = filterState.customChips.some(c => c.id === 'from-github-not');
      if (githubChipActive && email.sender === 'GitHub') return false;

      // 4. Search bar text filter
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        const matchesSender = email.sender.toLowerCase().includes(query);
        const matchesEmail = email.senderEmail.toLowerCase().includes(query);
        const matchesSubject = email.subject.toLowerCase().includes(query);
        const matchesBody = email.body.toLowerCase().includes(query);
        const matchesLabels = email.labels.some(l => l.toLowerCase().includes(query));
        
        if (!matchesSender && !matchesEmail && !matchesSubject && !matchesBody && !matchesLabels) {
          return false;
        }
      }

      return true;
    });
  }, [emails, currentView, selectedLabel, selectedCategory, filterState, searchQuery]);

  // List of IDs of currently visible filtered emails
  const visibleEmailIds = useMemo(() => {
    return filteredEmails.map(e => e.id);
  }, [filteredEmails]);

  // Checked email IDs that are currently visible
  const selectedVisibleIds = useMemo(() => {
    return visibleEmailIds.filter(id => !!checkedEmails[id]);
  }, [visibleEmailIds, checkedEmails]);

  const isAllSelected = useMemo(() => {
    return visibleEmailIds.length > 0 && visibleEmailIds.every(id => !!checkedEmails[id]);
  }, [visibleEmailIds, checkedEmails]);

  const isSomeSelected = useMemo(() => {
    return selectedVisibleIds.length > 0 && !isAllSelected;
  }, [selectedVisibleIds, isAllSelected]);

  const toggleSelectAll = () => {
    if (isAllSelected) {
      setCheckedEmails(prev => {
        const next = { ...prev };
        visibleEmailIds.forEach(id => {
          next[id] = false;
        });
        return next;
      });
    } else {
      setCheckedEmails(prev => {
        const next = { ...prev };
        visibleEmailIds.forEach(id => {
          next[id] = true;
        });
        return next;
      });
    }
  };

  const clearSelection = () => {
    setCheckedEmails({});
  };

  const executeBulkArchive = () => {
    if (selectedVisibleIds.length === 0) return;
    if (onBulkArchive) {
      onBulkArchive(selectedVisibleIds);
    } else if (onArchive) {
      selectedVisibleIds.forEach(id => onArchive(id));
    }
    clearSelection();
  };

  const executeBulkDelete = () => {
    if (selectedVisibleIds.length === 0) return;
    if (onBulkDelete) {
      onBulkDelete(selectedVisibleIds);
    } else if (onDelete) {
      selectedVisibleIds.forEach(id => onDelete(id));
    }
    clearSelection();
  };

  const executeBulkToggleUnread = (isUnread: boolean) => {
    if (selectedVisibleIds.length === 0) return;
    if (onBulkToggleUnread) {
      onBulkToggleUnread(selectedVisibleIds, isUnread);
    } else if (onToggleUnread) {
      selectedVisibleIds.forEach(id => {
        const email = emails.find(e => e.id === id);
        if (email && email.isUnread !== isUnread) {
          onToggleUnread(id);
        }
      });
    }
    clearSelection();
  };

  const executeBulkToggleStar = (isStarred: boolean) => {
    if (selectedVisibleIds.length === 0) return;
    if (onBulkToggleStar) {
      onBulkToggleStar(selectedVisibleIds, isStarred);
    } else if (onToggleStar) {
      selectedVisibleIds.forEach(id => {
        const email = emails.find(e => e.id === id);
        if (email && email.isStarred !== isStarred) {
          onToggleStar(id);
        }
      });
    }
    clearSelection();
  };

  // Group emails by relative date category
  const groupedEmails = useMemo(() => {
    const today: Email[] = [];
    const yesterday: Email[] = [];
    const last7Days: Email[] = [];
    const last30Days: Email[] = [];
    const older: Email[] = [];

    const now = new Date('2026-07-18T08:09:44-07:00'); // Standardized reference time from system metadata

    filteredEmails.forEach(email => {
      const emailDate = new Date(email.timestamp);
      
      // Difference in days
      const diffTime = now.getTime() - emailDate.getTime();
      const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

      const isSameDay = emailDate.getDate() === now.getDate() && 
                        emailDate.getMonth() === now.getMonth() && 
                        emailDate.getFullYear() === now.getFullYear();

      const yesterdayDate = new Date(now);
      yesterdayDate.setDate(now.getDate() - 1);
      const isYesterday = emailDate.getDate() === yesterdayDate.getDate() && 
                          emailDate.getMonth() === yesterdayDate.getMonth() && 
                          emailDate.getFullYear() === yesterdayDate.getFullYear();

      if (isSameDay) {
        today.push(email);
      } else if (isYesterday) {
        yesterday.push(email);
      } else if (diffDays <= 7) {
        last7Days.push(email);
      } else if (diffDays <= 30) {
        last30Days.push(email);
      } else {
        older.push(email);
      }
    });

    // Sort within each group descending by time
    const sortByTime = (a: Email, b: Email) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();

    return [
      { title: 'Today', emails: today.sort(sortByTime) },
      { title: 'Yesterday', emails: yesterday.sort(sortByTime) },
      { title: 'Last 7 days', emails: last7Days.sort(sortByTime) },
      { title: 'Last 30 days', emails: last30Days.sort(sortByTime) },
      { title: 'Older', emails: older.sort(sortByTime) }
    ].filter(group => group.emails.length > 0);
  }, [filteredEmails]);

  // Human-readable date display helper
  const formatTimeOrDate = (isoString: string) => {
    const date = new Date(isoString);
    const now = new Date('2026-07-18T08:09:44-07:00');
    
    const isSameDay = date.getDate() === now.getDate() && 
                      date.getMonth() === now.getMonth() && 
                      date.getFullYear() === now.getFullYear();

    if (isSameDay) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  // Convert HTML email body into plain snippet text
  const getBodySnippet = (htmlContent: string) => {
    const doc = new DOMParser().parseFromString(htmlContent, 'text/html');
    const text = doc.body.textContent || '';
    return text.trim().replace(/\s+/g, ' ').substring(0, 140);
  };

  // Human label helper for display
  const getFolderDisplayName = () => {
    if (currentView === 'inbox') return 'Inbox';
    if (currentView === 'unread') return 'Unread';
    if (currentView === 'all-mail') return 'All Mail';
    if (currentView === 'sent') return 'Sent';
    if (currentView === 'drafts') return 'Drafts';
    if (currentView === 'spam') return 'Spam';
    if (currentView === 'trash') return 'Trash';
    if (currentView === 'starred') return 'Starred';
    if (currentView === 'github') return 'GitHub';
    if (currentView === 'label') return `Label: ${selectedLabel}`;
    if (currentView === 'categories') return `Category: ${selectedCategory}`;
    return 'Inbox';
  };

  return (
    <div id="mail-list-panel" className="flex-1 h-screen flex flex-col bg-canvas-base overflow-hidden">
      
      {/* Header Panel */}
      {isSelectionMode ? (
        <div id="mail-list-header-bulk" className="px-6 py-5 flex items-center justify-between mb-1 bg-blue-500/5 border-b border-blue-500/10">
          <div className="flex items-center gap-3.5">
            {/* Master Checkbox */}
            <div
              id="master-checkbox"
              onClick={toggleSelectAll}
              title={isAllSelected ? "Deselect all" : "Select all"}
              className={`w-4 h-4 rounded border flex items-center justify-center transition-colors cursor-pointer bg-[#1c1b1b]/40 ${
                isAllSelected 
                  ? 'border-blue-500 bg-blue-500/10' 
                  : 'border-white/10 hover:border-white/30'
              }`}
            >
              {isAllSelected && <Check className="w-2.5 h-2.5 text-blue-500" />}
              {isSomeSelected && <div className="w-1.5 h-0.5 bg-blue-500 rounded" />}
            </div>
            
            <span className="text-sm font-medium text-text-primary select-none">
              {selectedVisibleIds.length} selected
            </span>
            
            <button
              id="btn-clear-selection"
              onClick={clearSelection}
              title="Clear selection"
              className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          {/* Bulk actions */}
          <div className="flex items-center gap-2">
            <button
              id="btn-bulk-unread"
              onClick={() => executeBulkToggleUnread(true)}
              title="Mark as unread"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <Mail className="w-4 h-4" />
            </button>
            
            <button
              id="btn-bulk-read"
              onClick={() => executeBulkToggleUnread(false)}
              title="Mark as read"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <MailOpen className="w-4 h-4" />
            </button>
            
            <button
              id="btn-bulk-star-on"
              onClick={() => executeBulkToggleStar(true)}
              title="Star"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <Star className="w-4 h-4" />
            </button>
            
            <button
              id="btn-bulk-star-off"
              onClick={() => executeBulkToggleStar(false)}
              title="Unstar"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <span className="relative flex items-center justify-center">
                <Star className="w-4 h-4 text-text-secondary" />
                <span className="absolute text-[8px] font-bold text-red-500/80 -mt-0.5">\</span>
              </span>
            </button>
            
            <button
              id="btn-bulk-archive"
              onClick={executeBulkArchive}
              title="Archive"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <Archive className="w-4 h-4" />
            </button>
            
            <button
              id="btn-bulk-delete"
              onClick={executeBulkDelete}
              title="Delete"
              className="p-1.5 rounded-lg hover:bg-red-500/10 text-red-400/80 hover:text-red-400 cursor-pointer transition-colors"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        </div>
      ) : (
        <div id="mail-list-header" className="px-6 py-5 flex items-center justify-between mb-1">
          <div className="flex items-center gap-2.5">
            {onToggleSidebar && (
              <button
                id="btn-toggle-sidebar"
                onClick={onToggleSidebar}
                className="md:hidden p-1.5 rounded-md hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors mr-1"
              >
                <Menu className="w-5 h-5" />
              </button>
            )}
            <Inbox className="w-5 h-5 text-text-primary" />
            <h1 className="text-base font-semibold text-text-primary select-none">{getFolderDisplayName()}</h1>
          </div>

          {/* Header Tools */}
          <div id="mail-header-tools" className="flex items-center gap-2">
            <button
              id="btn-filter-toggle"
              onClick={() => setMobileFiltersOpen(!mobileFiltersOpen)}
              title="Toggle filters"
              className={`p-1.5 rounded-lg cursor-pointer transition-colors ${
                mobileFiltersOpen 
                  ? 'bg-canvas-hover text-white' 
                  : 'hover:bg-canvas-hover text-text-secondary hover:text-text-primary'
              }`}
            >
              <ListFilter className="w-4 h-4" />
            </button>
            
            <button
              id="btn-refresh"
              onClick={onRefresh}
              title="Refresh inbox"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <RotateCw className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {/* Filter Control Bar */}
      <div 
        id="filter-bar" 
        className={`px-6 pb-4 bg-canvas-base items-center gap-2 select-none flex-wrap ${
          mobileFiltersOpen ? 'flex' : 'hidden'
        }`}
      >
        
        {/* Category Dropdown */}
        <div className="relative">
          <button
            id="filter-category-btn"
            onClick={() => {
              setCatDropdownOpen(!catDropdownOpen);
              setLabelDropdownOpen(false);
            }}
            className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-lg text-text-primary transition-colors ${
              filterState.category ? 'bg-canvas-hover text-white font-medium shadow-sm' : 'bg-canvas-card text-text-secondary hover:text-text-primary'
            }`}
          >
            <span>Category: {filterState.category || '[Gmail]'}</span>
            <ChevronDown className="w-3 h-3 text-text-secondary" />
          </button>
          
          {catDropdownOpen && (
            <div id="cat-dropdown" className="absolute left-0 mt-1.5 w-44 bg-canvas-modal rounded-lg shadow-2xl z-50 py-1">
              {allCategories.map(cat => (
                <button
                  key={cat}
                  id={`cat-filter-option-${cat}`}
                  onClick={() => selectCategoryFilter(cat)}
                  className="w-full text-left px-3 py-1.5 text-xs text-text-primary hover:bg-canvas-hover flex items-center justify-between"
                >
                  <span>{cat}</span>
                  {((cat === 'All' && !filterState.category) || filterState.category === cat) && (
                    <Check className="w-3.5 h-3.5 text-white" />
                  )}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Labels Dropdown */}
        <div className="relative">
          <button
            id="filter-labels-btn"
            onClick={() => {
              setLabelDropdownOpen(!labelDropdownOpen);
              setCatDropdownOpen(false);
            }}
            className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-lg text-text-primary transition-colors ${
              filterState.label ? 'bg-canvas-hover text-white font-medium shadow-sm' : 'bg-canvas-card text-text-secondary hover:text-text-primary'
            }`}
          >
            <span>Labels</span>
            <ChevronDown className="w-3 h-3 text-text-secondary" />
          </button>
          
          {labelDropdownOpen && (
            <div id="label-dropdown" className="absolute left-0 mt-1.5 w-48 bg-canvas-modal rounded-lg shadow-2xl z-50 py-1 max-h-60 overflow-y-auto">
              {allLabels.map(lbl => (
                <button
                  key={lbl}
                  id={`label-filter-option-${lbl}`}
                  onClick={() => selectLabelFilter(lbl)}
                  className="w-full text-left px-3 py-1.5 text-xs text-text-primary hover:bg-canvas-hover flex items-center justify-between"
                >
                  <span className="truncate">{lbl}</span>
                  {((lbl === 'All' && !filterState.label) || filterState.label === lbl) && (
                    <Check className="w-3.5 h-3.5 text-white" />
                  )}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Unread Toggle Chip */}
        <button
          id="toggle-filter-unread"
          onClick={() => setFilterState(prev => ({ ...prev, isUnread: !prev.isUnread }))}
          className={`px-3 py-1.5 text-xs font-mono rounded-full transition-colors ${
            filterState.isUnread 
              ? 'bg-canvas-hover text-white font-medium shadow-sm' 
              : 'bg-canvas-card text-text-secondary hover:text-text-primary'
          }`}
        >
          Is unread
        </button>

        {/* Show Archived Toggle Chip */}
        <button
          id="toggle-filter-archived"
          onClick={() => setFilterState(prev => ({ ...prev, showArchived: !prev.showArchived }))}
          className={`px-3 py-1.5 text-xs font-mono rounded-full transition-colors ${
            filterState.showArchived 
              ? 'bg-canvas-hover text-white font-medium shadow-sm' 
              : 'bg-canvas-card text-text-secondary hover:text-text-primary'
          }`}
        >
          Show archived
        </button>

        {/* Render Custom Preloaded Chips (e.g. From: Not "github.com") */}
        {filterState.customChips.map(chip => (
          <div
            key={chip.id}
            id={`custom-chip-${chip.id}`}
            className="flex items-center gap-1.5 bg-canvas-hover/90 hover:bg-canvas-hover pl-2.5 pr-2 py-1 rounded-full text-xs font-mono text-text-primary shadow-sm"
          >
            <span>{chip.field}: Not "{chip.value}"</span>
            <button
              id={`remove-chip-${chip.id}`}
              onClick={() => removeCustomChip(chip.id)}
              className="p-0.5 rounded-full hover:bg-canvas-base text-text-secondary hover:text-text-primary cursor-pointer"
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        ))}

        {/* Plus Filter Button (Simulated adding custom filters) */}
        <button
          id="btn-add-filter"
          onClick={() => {
            // Adds or toggles the GitHub exclusionary chip as an interactive demo
            const hasGitHubChip = filterState.customChips.some(c => c.id === 'from-github-not');
            if (!hasGitHubChip) {
              setFilterState(prev => ({
                ...prev,
                customChips: [...prev.customChips, { id: 'from-github-not', field: 'From', value: 'github.com' }]
              }));
            } else {
              setFilterState(prev => ({
                ...prev,
                customChips: prev.customChips.filter(c => c.id !== 'from-github-not')
              }));
            }
          }}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono text-text-secondary hover:text-text-primary bg-canvas-card rounded-full cursor-pointer transition-colors shadow-sm"
        >
          <Plus className="w-3 h-3" />
          <span>Filter</span>
        </button>

        {/* Clear Filters (if any are active) */}
        {(filterState.category || filterState.label || filterState.isUnread || filterState.showArchived || filterState.customChips.length > 0) && (
          <button
            id="btn-clear-filters"
            onClick={resetFilters}
            className="text-[11px] font-mono text-white/60 hover:text-white underline cursor-pointer ml-auto"
          >
            Clear all
          </button>
        )}
      </div>

      {/* Mail Items List scroll block */}
      <div id="mail-list-scrollable" className="flex-1 overflow-y-auto">
        {groupedEmails.length === 0 ? (
          <div id="empty-mail-state" className="flex flex-col items-center justify-center h-64 text-text-secondary select-none p-6">
            <MailWarning className="w-8 h-8 opacity-40 mb-2" />
            <span className="text-xs font-mono">No matching emails found</span>
            <button 
              id="empty-state-reset-btn"
              onClick={resetFilters}
              className="mt-4 px-4 py-2 bg-canvas-hover rounded-md text-xs text-text-primary cursor-pointer hover:bg-canvas-hover/80 transition-colors"
            >
              Reset Filters
            </button>
          </div>
        ) : (
          <div id="grouped-mail-container" className={`py-2 ${mailDenseMode ? 'space-y-1.5' : 'space-y-4 md:space-y-5'}`}>
            {groupedEmails.map(group => (
              <div key={group.title} id={`date-group-${group.title}`} className={mailDenseMode ? 'space-y-0.5' : 'space-y-1'}>
                {/* Date Header Section */}
                <div className={`px-6 py-1 text-xs font-medium text-text-secondary/70 bg-canvas-base/80 sticky top-0 z-10 backdrop-blur-sm select-none ${
                  mailDenseMode ? 'pb-0.5' : 'pb-1.5'
                }`}>
                  {group.title}
                </div>

                {/* Email Rows Table */}
                <div className="space-y-0.5 px-3">
                  {group.emails.map(email => (
                    <div
                      key={email.id}
                      id={`email-row-${email.id}`}
                      onClick={(e) => {
                        if (justLongPressedRef.current) {
                          justLongPressedRef.current = false;
                          return;
                        }
                        if (isSelectionMode) {
                          toggleCheck(e, email.id);
                        } else {
                          onEmailClick(email);
                        }
                      }}
                      onContextMenu={(e) => {
                        e.preventDefault();
                        setEmailContextMenu({
                          x: e.clientX,
                          y: e.clientY,
                          emailId: email.id
                        });
                      }}
                      onTouchStart={(e) => handleTouchStart(e, email.id)}
                      onTouchEnd={handleTouchEnd}
                      onTouchMove={handleTouchMove}
                      className={`group flex items-center px-4 bg-canvas-base hover:bg-canvas-hover rounded-lg cursor-pointer select-none transition-none relative ${
                        mailDenseMode ? 'py-0.5 md:py-1' : 'py-1 md:py-1.5'
                      }`}
                    >
                      {/* Checkbox */}
                      <div className={`mr-3 shrink-0 items-center justify-center ${
                        isSelectionMode ? 'flex' : 'hidden md:flex'
                      }`}>
                        <div
                          id={`email-checkbox-${email.id}`}
                          onClick={(e) => toggleCheck(e, email.id)}
                          className={`w-4 h-4 rounded border flex items-center justify-center transition-colors cursor-pointer bg-[#1c1b1b]/40 ${
                            checkedEmails[email.id] 
                              ? 'border-blue-500 bg-blue-500/10' 
                              : 'border-white/10 hover:border-white/30'
                          }`}
                        >
                          {checkedEmails[email.id] && (
                            <Check className="w-2.5 h-2.5 text-blue-500" />
                          )}
                        </div>
                      </div>

                      {/* Left indicator blue dot */}
                      <div className="w-4 flex items-center justify-start shrink-0 mr-1.5">
                        {email.isUnread && (
                          <span 
                            id={`unread-dot-${email.id}`} 
                            className="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]" 
                          />
                        )}
                      </div>

                      {/* Sender Column */}
                      <div className="w-24 sm:w-36 md:w-48 pr-2 sm:pr-4 shrink-0 font-medium text-text-primary text-xs sm:text-sm truncate">
                        {email.sender}
                      </div>

                      {/* Subject + Body preview column */}
                      <div className="flex-1 pr-4 flex items-center gap-1 sm:gap-2 overflow-hidden text-xs sm:text-sm">
                        <span className={`text-text-primary truncate shrink-0 max-w-[80px] sm:max-w-[150px] md:max-w-xs lg:max-w-md ${email.isUnread ? 'font-semibold' : 'font-normal'}`}>
                          {email.subject}
                        </span>
                        <span className="text-text-secondary/80 truncate text-[11px] sm:text-xs font-light hidden sm:inline flex-1">
                          — {getBodySnippet(email.body)}
                        </span>
                      </div>

                      {/* Right metadata / Actions panel on Hover */}
                      <div className="ml-auto shrink-0 flex items-center justify-end gap-3 relative h-full select-none min-h-[28px]">
                        {/* Static view: Labels + Attachment + Date (hidden on hover) */}
                        <div className="flex items-center gap-2 sm:gap-3 text-xs text-text-secondary group-hover:opacity-0 transition-opacity duration-150">
                          {/* Custom Label Pills - Desktop only, hidden on hover */}
                          {email.labels && email.labels.length > 0 && (
                            <div className="hidden md:flex items-center gap-1.5 select-none overflow-hidden max-w-[150px] lg:max-w-[300px] xl:max-w-[450px]">
                              {email.labels.map(label => {
                                const labelMeta = getLabelStyle(label);
                                const LabelIcon = labelMeta.icon;
                                return (
                                  <span 
                                    key={label}
                                    className={`inline-flex items-center gap-0.5 px-2 py-0.5 rounded-full text-[10px] font-medium border ${labelMeta.bgColor} ${labelMeta.textColor} ${labelMeta.borderColor} truncate`}
                                    title={label}
                                  >
                                    <LabelIcon className="w-3 h-3 shrink-0" />
                                    <span className="truncate">{label}</span>
                                  </span>
                                );
                              })}
                            </div>
                          )}

                          {/* Attachment & Date */}
                          <div className="flex items-center gap-1 sm:gap-2.5 font-mono text-[11px] sm:text-xs text-text-secondary/80 shrink-0">
                            {email.hasAttachment && (
                              <Paperclip className="w-3.5 h-3.5 text-text-secondary/60 shrink-0" />
                            )}
                            <span className="truncate">{formatTimeOrDate(email.timestamp)}</span>
                          </div>
                        </div>

                        {/* Hover: Custom actions overlay styled as a modern floating card */}
                        <div 
                          className="absolute right-0 top-1/2 -translate-y-1/2 hidden group-hover:flex items-center gap-1 bg-[#1a1919] border border-white/10 rounded-md p-1 shadow-2xl animate-fade-in"
                          onClick={(e) => e.stopPropagation()}
                        >
                          {/* Toggle Star */}
                          <button
                            id={`btn-row-star-${email.id}`}
                            title="Star"
                            onClick={() => {
                              if (onToggleStar) onToggleStar(email.id);
                            }}
                            className={`p-1.5 rounded hover:bg-white/10 text-text-secondary hover:text-white transition-colors cursor-pointer ${
                              email.isStarred ? 'text-yellow-400 hover:text-yellow-300' : ''
                            }`}
                          >
                            <Star className={`w-3.5 h-3.5 ${email.isStarred ? 'fill-current' : ''}`} />
                          </button>

                          {/* Archive */}
                          <button
                            id={`btn-row-archive-${email.id}`}
                            title="Archive"
                            onClick={() => {
                              if (onArchive) onArchive(email.id);
                            }}
                            className="p-1.5 rounded hover:bg-white/10 text-text-secondary hover:text-white transition-colors cursor-pointer"
                          >
                            <Archive className="w-3.5 h-3.5" />
                          </button>

                          {/* Trash / Delete with slight hover color */}
                          <button
                            id={`btn-row-delete-${email.id}`}
                            title="Delete"
                            onClick={() => {
                              if (onDelete) onDelete(email.id);
                            }}
                            className="p-1.5 rounded hover:bg-red-500/10 text-text-secondary hover:text-red-400 transition-colors cursor-pointer"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>

                          {/* CheckSquare / Read toggle */}
                          <button
                            id={`btn-row-read-${email.id}`}
                            title={email.isUnread ? "Mark as Read" : "Mark as Unread"}
                            onClick={() => {
                              if (onToggleUnread) onToggleUnread(email.id);
                            }}
                            className="p-1.5 rounded hover:bg-white/10 text-text-secondary hover:text-white transition-colors cursor-pointer"
                          >
                            <CheckSquare className="w-3.5 h-3.5" />
                          </button>

                          {/* Snooze (Clock) */}
                          <button
                            id={`btn-row-snooze-${email.id}`}
                            title="Snooze"
                            onClick={() => {
                              if (showToast) {
                                showToast(`Snoozed thread until tomorrow morning.`, "success");
                              } else {
                                alert("Snoozed thread until tomorrow morning.");
                              }
                            }}
                            className="p-1.5 rounded hover:bg-white/10 text-text-secondary hover:text-white transition-colors cursor-pointer"
                          >
                            <Clock className="w-3.5 h-3.5" />
                          </button>

                          {/* Tag / Custom Label */}
                          <button
                            id={`btn-row-label-${email.id}`}
                            title="Label"
                            onClick={() => {
                              const customLabel = prompt("Enter a label for this thread:");
                              if (customLabel && customLabel.trim()) {
                                if (onAddLabel) onAddLabel(email.id, customLabel.trim());
                                if (showToast) showToast(`Added label "${customLabel.trim()}" successfully.`, "success");
                              }
                            }}
                            className="p-1.5 rounded hover:bg-white/10 text-text-secondary hover:text-white transition-colors cursor-pointer"
                          >
                            <Tag className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Reusable Custom Email Context Menu */}
      {emailContextMenu && (() => {
        const targetEmail = emails.find(e => e.id === emailContextMenu.emailId);
        if (!targetEmail) return null;
        return (
          <>
            <div 
              id="email-context-backdrop"
              className="fixed inset-0 z-50 cursor-default" 
              onClick={() => setEmailContextMenu(null)}
              onContextMenu={(e) => {
                e.preventDefault();
                setEmailContextMenu(null);
              }}
            />
            <div 
              id="email-context-menu"
              className="fixed bg-[#1a1919] border border-white/10 rounded-lg shadow-2xl py-1 w-48 z-50 font-sans"
              style={{ 
                top: emailContextMenu.y, 
                left: Math.min(emailContextMenu.x, window.innerWidth - 200) 
              }}
            >
              <div className="px-3 py-1.5 text-[10px] font-mono uppercase tracking-wider text-text-secondary border-b border-white/5 mb-1 truncate">
                Actions
              </div>

              <button
                id="context-email-unread"
                onClick={() => {
                  if (onToggleUnread) onToggleUnread(targetEmail.id);
                  setEmailContextMenu(null);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center gap-2 text-left cursor-pointer transition-colors"
              >
                {targetEmail.isUnread ? (
                  <>
                    <MailOpen className="w-3.5 h-3.5 text-blue-400" />
                    <span>Mark as Read</span>
                  </>
                ) : (
                  <>
                    <Mail className="w-3.5 h-3.5 text-blue-400" />
                    <span>Mark as Unread</span>
                  </>
                )}
              </button>

              <button
                id="context-email-star"
                onClick={() => {
                  if (onToggleStar) onToggleStar(targetEmail.id);
                  setEmailContextMenu(null);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center gap-2 text-left cursor-pointer transition-colors"
              >
                <Star className={`w-3.5 h-3.5 ${targetEmail.isStarred ? 'text-yellow-400 fill-yellow-400' : 'text-text-secondary'}`} />
                <span>{targetEmail.isStarred ? 'Remove Star' : 'Star Thread'}</span>
              </button>

              <button
                id="context-email-archive"
                onClick={() => {
                  if (onArchive) onArchive(targetEmail.id);
                  setEmailContextMenu(null);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center gap-2 text-left cursor-pointer transition-colors"
              >
                <Archive className="w-3.5 h-3.5 text-green-400" />
                <span>Archive</span>
              </button>

              {/* Tag / Label option */}
              <button
                id="context-email-label"
                onClick={() => {
                  setEmailContextMenu(null);
                  setTimeout(() => {
                    const customLabel = prompt("Enter a label for this thread:");
                    if (customLabel && customLabel.trim()) {
                      if (onAddLabel) onAddLabel(targetEmail.id, customLabel.trim());
                      if (showToast) showToast(`Added label "${customLabel.trim()}" successfully.`, "success");
                    }
                  }, 100);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center gap-2 text-left cursor-pointer transition-colors"
              >
                <Tag className="w-3.5 h-3.5 text-purple-400" />
                <span>Add Label...</span>
              </button>

              <button
                id="context-email-delete"
                onClick={() => {
                  if (onDelete) onDelete(targetEmail.id);
                  setEmailContextMenu(null);
                }}
                className="w-full px-3 py-2 text-xs text-red-400 hover:bg-red-500/10 flex items-center gap-2 text-left cursor-pointer transition-colors border-t border-white/5 mt-1"
              >
                <Trash2 className="w-3.5 h-3.5 text-red-400" />
                <span>Delete</span>
              </button>
            </div>
          </>
        );
      })()}

    </div>
  );
}
