import React, { useState } from 'react';
import { 
  Inbox, 
  Tag, 
  CircleDot, 
  Github, 
  Folder, 
  Plus, 
  Mail, 
  Send, 
  FileText, 
  AlertTriangle, 
  Trash2, 
  Settings, 
  MessageSquare, 
  HelpCircle, 
  Search, 
  ChevronDown, 
  ChevronRight,
  PenSquare,
  X,
  Check,
  Edit2,
  Calendar,
  ToggleLeft,
  ToggleRight
} from 'lucide-react';
import { Email, ViewType } from '../types';
import { getLabelConfig, iconMapping, colorConfigs } from '../data/labelConfig';

interface SidebarProps {
  emails: Email[];
  currentView: ViewType;
  setView: (view: ViewType) => void;
  selectedLabel: string | null;
  setSelectedLabel: (label: string | null) => void;
  selectedCategory: string | null;
  setSelectedCategory: (cat: string | null) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  onComposeClick: () => void;
  isMobileOpen?: boolean;
  onCloseMobile?: () => void;
  
  // Custom label configurations & actions
  labelCustomizations: Record<string, { iconName: string; colorName: string }>;
  onRenameLabel: (oldName: string, newName: string) => void;
  onUpdateLabelCustomization: (label: string, iconName: string, colorName: string) => void;
  onDeleteLabel: (label: string) => void;

  currentPath?: string;
  onNavigate?: (path: string) => void;

  // Mail settings props
  mailDenseMode: boolean;
  setMailDenseMode: (dense: boolean) => void;
  mailDefaultLandingView: string;
  setMailDefaultLandingView: (view: string) => void;
  mailSignature: string;
  setMailSignature: (sig: string) => void;
}

export default function Sidebar({
  emails,
  currentView,
  setView,
  selectedLabel,
  setSelectedLabel,
  selectedCategory,
  setSelectedCategory,
  searchQuery,
  setSearchQuery,
  onComposeClick,
  isMobileOpen,
  onCloseMobile,
  labelCustomizations,
  onRenameLabel,
  onUpdateLabelCustomization,
  onDeleteLabel,
  currentPath = '/',
  onNavigate,
  mailDenseMode,
  setMailDenseMode,
  mailDefaultLandingView,
  setMailDefaultLandingView,
  mailSignature,
  setMailSignature
}: SidebarProps) {
  const [labelsExpanded, setLabelsExpanded] = useState(false);
  const [categoriesExpanded, setCategoriesExpanded] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Context Menu and editing states
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; label: string } | null>(null);
  const [editingLabel, setEditingLabel] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [editIcon, setEditIcon] = useState('Tag');
  const [editColor, setEditColor] = useState('blue');
  const [showColorsDropdown, setShowColorsDropdown] = useState(false);
  const [showIconsDropdown, setShowIconsDropdown] = useState(false);
  const [isDeletingConfirmOpen, setIsDeletingConfirmOpen] = useState(false);
  const [showNestingDropdown, setShowNestingDropdown] = useState(false);
  const [collapsedLabels, setCollapsedLabels] = useState<Record<string, boolean>>(() => {
    try {
      const saved = localStorage.getItem('collapsed_labels');
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  });

  // Helper to build recursive label tree from flat string paths
  interface LabelNode {
    name: string;
    displayName: string;
    children: LabelNode[];
    isReal: boolean;
  }

  const buildLabelTree = (labels: string[]): LabelNode[] => {
    const root: LabelNode[] = [];
    
    labels.forEach(fullPath => {
      const parts = fullPath.split('/');
      let currentLevel = root;
      
      parts.forEach((part, index) => {
        const pathSoFar = parts.slice(0, index + 1).join('/');
        let node = currentLevel.find(n => n.displayName === part);
        
        if (!node) {
          node = {
            name: pathSoFar,
            displayName: part,
            children: [],
            isReal: pathSoFar === fullPath
          };
          currentLevel.push(node);
        } else {
          if (pathSoFar === fullPath) {
            node.isReal = true;
          }
        }
        currentLevel = node.children;
      });
    });
    
    return root;
  };

  const toggleLabelCollapse = (labelName: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setCollapsedLabels(prev => {
      const updated = {
        ...prev,
        [labelName]: !prev[labelName]
      };
      localStorage.setItem('collapsed_labels', JSON.stringify(updated));
      return updated;
    });
  };

  interface FlattenedLabelItem {
    name: string;
    displayName: string;
    depth: number;
    hasChildren: boolean;
    isExpanded: boolean;
    isReal: boolean;
  }

  const getFlattenedLabels = (): FlattenedLabelItem[] => {
    const tree = buildLabelTree(allLabels);
    const result: FlattenedLabelItem[] = [];

    const traverse = (nodes: LabelNode[], depth: number) => {
      // Sort nodes alphabetically
      const sortedNodes = [...nodes].sort((a, b) => a.displayName.localeCompare(b.displayName));
      
      sortedNodes.forEach(node => {
        const hasChildren = node.children.length > 0;
        const isCollapsed = !!collapsedLabels[node.name];
        
        result.push({
          name: node.name,
          displayName: node.displayName,
          depth,
          hasChildren,
          isExpanded: !isCollapsed,
          isReal: node.isReal
        });

        if (hasChildren && !isCollapsed) {
          traverse(node.children, depth + 1);
        }
      });
    };

    traverse(tree, 0);
    return result;
  };

  const getEmailCountForLabel = (labelPath: string) => {
    return emails.filter(e => 
      !e.isTrash && 
      e.labels.some(l => l === labelPath || l.startsWith(labelPath + '/'))
    ).length;
  };

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

  // Dynamic state calculations
  const inboxCount = emails.filter(e => !e.isArchived && !e.isTrash && !e.isSpam && !e.isDraft).length;
  const unreadCount = emails.filter(e => e.isUnread && !e.isArchived && !e.isTrash && !e.isSpam).length;
  const draftCount = emails.filter(e => e.isDraft).length;
  const githubCount = emails.filter(e => e.sender === 'GitHub' && !e.isTrash).length;

  // Extract unique labels
  const allLabels = Array.from(new Set(emails.flatMap(e => e.labels)));
  const labelCounts = allLabels.reduce((acc, label) => {
    acc[label] = emails.filter(e => e.labels.includes(label) && !e.isTrash).length;
    return acc;
  }, {} as Record<string, number>);

  // Extract unique categories
  const allCategories = ['Primary', 'Updates', 'Social', 'Promotions', 'Forums'];
  const categoryCounts = allCategories.reduce((acc, cat) => {
    acc[cat] = emails.filter(e => e.category === cat && !e.isTrash && !e.isArchived).length;
    return acc;
  }, {} as Record<string, number>);

  const handleViewSelect = (view: ViewType) => {
    if (onNavigate && currentPath !== '/') {
      onNavigate('/');
    }
    setView(view);
    setSelectedLabel(null);
    setSelectedCategory(null);
    if (onCloseMobile) onCloseMobile();
  };

  const handleLabelSelect = (label: string) => {
    if (onNavigate && currentPath !== '/') {
      onNavigate('/');
    }
    setView('label');
    setSelectedLabel(label);
    setSelectedCategory(null);
    if (onCloseMobile) onCloseMobile();
  };

  const handleCategorySelect = (category: string) => {
    if (onNavigate && currentPath !== '/') {
      onNavigate('/');
    }
    setView('categories');
    setSelectedCategory(category);
    setSelectedLabel(null);
    if (onCloseMobile) onCloseMobile();
  };

  return (
    <div 
      id="sidebar-container" 
      className={`fixed md:relative top-0 bottom-0 left-0 z-40 w-64 h-screen bg-canvas-card flex flex-col font-sans select-none transition-transform duration-200 md:translate-x-0 ${
        isMobileOpen ? 'translate-x-0' : '-translate-x-full'
      }`}
    >
      {/* Profile Header */}
      <div id="profile-section" className="p-4 flex items-center justify-between mb-2">
        <div className="flex items-center gap-3 overflow-hidden">
          <div className="w-8 h-8 rounded-lg bg-white/10 flex items-center justify-center font-mono text-sm font-semibold text-white">
            D
          </div>
          <div className="flex flex-col overflow-hidden">
            <span className="text-sm font-medium text-text-primary truncate">Developer</span>
            <span className="text-xs text-text-secondary truncate">user@workspace.io</span>
          </div>
        </div>
        
        <div className="flex items-center gap-1">
          {/* Compose Button Icon */}
          <button 
            id="btn-compose-sidebar"
            onClick={() => {
              onComposeClick();
              if (onCloseMobile) onCloseMobile();
            }}
            title="Compose Message (C)"
            className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
          >
            <PenSquare className="w-4 h-4" />
          </button>
          
          {/* Close Sidebar Icon (mobile only) */}
          {onCloseMobile && (
            <button
              id="btn-close-sidebar-mobile"
              onClick={onCloseMobile}
              className="md:hidden p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {/* Interactive Search */}
      <div id="search-section" className="px-4 mb-4">
        <div className="relative flex-1">
          <Search className="w-3.5 h-3.5 text-text-secondary absolute left-2.5 top-2.5" />
          <input
            id="sidebar-search-input"
            type="text"
            placeholder="Search mail..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-canvas-base hover:bg-canvas-hover/40 focus:bg-canvas-hover text-text-primary text-xs rounded-lg pl-8 pr-8 py-1.5 outline-none border border-border-hairline focus:border-white/20 transition-all placeholder:text-text-secondary/40"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-2 text-text-secondary/50 hover:text-text-primary p-0.5 rounded cursor-pointer animate-fade-in"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>

      {/* Main Nav Scroll Area */}
      <div id="sidebar-nav-scroll" className="flex-1 overflow-y-auto px-2 space-y-6">
        
        {/* Views Section */}
        <div>
          <div className="px-2.5 py-1 text-[11px] font-mono tracking-wider text-text-secondary/70 uppercase">
            Views
          </div>
          <div className="space-y-0.5 mt-2">
            <button
              id="view-inbox"
              onClick={() => handleViewSelect('inbox')}
              className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'inbox' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <div className="flex items-center gap-2">
                <Inbox className="w-4 h-4 text-blue-400" />
                <span>Inbox</span>
              </div>
              {inboxCount > 0 && (
                <span className="font-mono text-[10px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                  {inboxCount}
                </span>
              )}
            </button>

            <button
              id="view-unread"
              onClick={() => handleViewSelect('unread')}
              className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'unread' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <div className="flex items-center gap-2">
                <CircleDot className="w-4 h-4 text-emerald-400" />
                <span>Unread</span>
              </div>
              {unreadCount > 0 && (
                <span className="font-mono text-[10px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                  {unreadCount}
                </span>
              )}
            </button>

            <button
              id="view-sent"
              onClick={() => handleViewSelect('sent')}
              className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'sent' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <Send className="w-4 h-4 text-violet-400" />
              <span>Sent</span>
            </button>

            <button
              id="view-drafts"
              onClick={() => handleViewSelect('drafts')}
              className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'drafts' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <div className="flex items-center gap-2">
                <FileText className="w-4 h-4 text-amber-400" />
                <span>Drafts</span>
              </div>
              {draftCount > 0 && (
                <span className="font-mono text-[10px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                  {draftCount}
                </span>
              )}
            </button>

            <button
              id="view-github"
              onClick={() => handleViewSelect('github')}
              className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'github' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <div className="flex items-center gap-2">
                <Github className="w-4 h-4 text-indigo-400" />
                <span>GitHub</span>
              </div>
              {githubCount > 0 && (
                <span className="font-mono text-[10px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                  {githubCount}
                </span>
              )}
            </button>

            {/* Categories Trigger */}
            <div>
              <button
                id="view-categories-toggle"
                onClick={() => setCategoriesExpanded(!categoriesExpanded)}
                className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium text-text-primary hover:bg-canvas-hover/60 transition-colors`}
              >
                <div className="flex items-center gap-2">
                  <Folder className="w-4 h-4 text-teal-400" />
                  <span>Categories</span>
                </div>
                <div className="flex items-center gap-1">
                  <span className="font-mono text-[10px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                    {allCategories.length}
                  </span>
                  {categoriesExpanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                </div>
              </button>

              {categoriesExpanded && (
                <div className="pl-6 pr-1 py-1 space-y-0.5 ml-4.5 mt-0.5">
                  {allCategories.map(cat => (
                    <button
                      key={cat}
                      id={`sidebar-category-${cat}`}
                      onClick={() => handleCategorySelect(cat)}
                      className={`w-full flex items-center justify-between px-2 py-1 rounded-md text-[11px] transition-colors ${
                        currentView === 'categories' && selectedCategory === cat
                          ? 'bg-canvas-hover text-white' 
                          : 'text-text-secondary hover:text-text-primary hover:bg-canvas-hover/40'
                      }`}
                    >
                      <span>{cat}</span>
                      <span className="font-mono text-[9px] text-text-secondary/70">
                        {categoryCounts[cat] || 0}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </div>

            <button
              id="view-all-mail"
              onClick={() => handleViewSelect('all-mail')}
              className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'all-mail' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <Mail className="w-4 h-4 text-pink-400" />
              <span>All Mail</span>
            </button>

            <button
              id="view-spam"
              onClick={() => handleViewSelect('spam')}
              className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'spam' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <AlertTriangle className="w-4 h-4 text-orange-400" />
              <span>Spam</span>
            </button>

            <button
              id="view-trash"
              onClick={() => handleViewSelect('trash')}
              className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                currentView === 'trash' 
                  ? 'bg-canvas-hover text-white' 
                  : 'text-text-primary hover:bg-canvas-hover/60'
              }`}
            >
              <Trash2 className="w-4 h-4 text-red-400" />
              <span>Trash</span>
            </button>
            
            {/* Mock Add View */}
            <button
              id="view-add-view"
              onClick={() => alert("Custom views feature is a work in progress! Stay tuned.")}
              className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs text-text-secondary hover:bg-canvas-hover/40 transition-colors cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              <span>Add view</span>
            </button>
          </div>
        </div>

        {/* Labels Section (Where we actually see the labels) */}
        <div>
          <div className="px-2.5 py-1 text-[11px] font-mono tracking-wider text-text-secondary/70 uppercase">
            Labels
          </div>
          <div className="space-y-0.5 mt-2 text-left">
            {getFlattenedLabels().map(item => {
              const labelMeta = getLabelStyle(item.name);
              const LabelIcon = labelMeta.icon;
              const isSelected = currentView === 'label' && selectedLabel === item.name;
              const emailCount = getEmailCountForLabel(item.name);

              return (
                <div
                  key={item.name}
                  className="group relative flex items-center w-full"
                  style={{ paddingLeft: `${item.depth * 12}px` }}
                >
                  {/* Expand/Collapse Chevron if has children */}
                  {item.hasChildren ? (
                    <button
                      type="button"
                      onClick={(e) => toggleLabelCollapse(item.name, e)}
                      className="absolute p-0.5 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer z-10"
                      style={{ left: `${item.depth * 12 + 2}px` }}
                    >
                      {item.isExpanded ? (
                        <ChevronDown className="w-3.5 h-3.5" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5" />
                      )}
                    </button>
                  ) : null}

                  <button
                    id={`sidebar-label-${item.name}`}
                    onClick={() => handleLabelSelect(item.name)}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      const custom = labelCustomizations?.[item.name];
                      const base = getLabelConfig(item.name);
                      const labelIconName = custom?.iconName || 'Tag';
                      const labelColorName = custom?.colorName || base.color;
                      
                      const lastSlashIndex = item.name.lastIndexOf('/');
                      const segmentName = lastSlashIndex !== -1 ? item.name.substring(lastSlashIndex + 1) : item.name;

                      setEditName(segmentName);
                      setEditIcon(labelIconName);
                      setEditColor(labelColorName);
                      setShowColorsDropdown(false);
                      setShowIconsDropdown(false);
                      setShowNestingDropdown(false);
                      setContextMenu({
                        x: e.clientX,
                        y: e.clientY,
                        label: item.name
                      });
                    }}
                    className={`w-full flex items-center justify-between py-1.5 pr-2.5 rounded-lg text-xs font-medium transition-all ${
                      item.hasChildren ? 'pl-7' : 'pl-6'
                    } ${
                      isSelected
                        ? 'bg-canvas-hover text-white font-semibold' 
                        : 'text-text-primary hover:bg-canvas-hover/60'
                    }`}
                  >
                    <div className="flex items-center gap-2 truncate">
                      <LabelIcon className={`w-3.5 h-3.5 shrink-0 ${labelMeta.textColor}`} />
                      <span className="truncate text-text-primary">{item.displayName}</span>
                    </div>
                    {emailCount > 0 && (
                      <span className="font-mono text-[9px] text-text-secondary bg-canvas-base px-1.5 py-0.5 rounded-md">
                        {emailCount}
                      </span>
                    )}
                  </button>
                </div>
              );
            })}
          </div>
        </div>

      </div>

      {/* Footer Controls */}
      <div id="sidebar-footer" className="p-3 bg-canvas-base flex flex-col gap-1 text-text-secondary mt-auto">
        <button 
          id="btn-settings-sidebar"
          onClick={() => setIsSettingsOpen(true)}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-canvas-hover/60 transition-colors text-left cursor-pointer"
        >
          <Settings className="w-4 h-4" />
          <span>Settings</span>
        </button>
      </div>

      {/* MAIL SETTINGS MODAL OVERLAY */}
      {isSettingsOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs animate-fade-in">
          <div className="fixed inset-0 cursor-pointer" onClick={() => setIsSettingsOpen(false)} />
          
          <div className="relative w-full max-w-md bg-[#131313] border border-neutral-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden animate-scale-in z-50">
            {/* Modal Header */}
            <div className="px-5 py-4 border-b border-neutral-800/60 flex items-center justify-between bg-[#181818]">
              <div className="flex items-center gap-2">
                <Settings className="w-4 h-4 text-blue-400 animate-spin-slow" />
                <span className="text-xs font-mono font-semibold text-text-primary uppercase tracking-wider">
                  Mail Settings
                </span>
              </div>
              <button 
                onClick={() => setIsSettingsOpen(false)}
                className="p-1 rounded-md hover:bg-neutral-800 text-text-secondary hover:text-text-primary transition-all cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-5 space-y-6 overflow-y-auto max-h-[80vh] scrollbar-thin">
              
              {/* Option A: Default Landing View */}
              <div className="space-y-2.5">
                <label className="block text-[10px] font-mono text-text-secondary/60 uppercase tracking-wider">
                  Default landing View
                </label>
                <div className="space-y-2">
                  {[
                    { id: 'inbox', name: 'Inbox' },
                    { id: 'unread', name: 'Unread' },
                    { id: 'all-mail', name: 'All Mail' }
                  ].map(v => {
                    const isSelected = mailDefaultLandingView === v.id;
                    return (
                      <button
                        key={v.id}
                        onClick={() => {
                          setMailDefaultLandingView(v.id);
                          localStorage.setItem('kestrel_mail_default_view', v.id);
                        }}
                        className={`w-full flex items-center justify-between p-3 rounded-xl border text-xs transition-all cursor-pointer ${
                          isSelected 
                            ? 'bg-[#1a1a1a] border-blue-500/50 text-text-primary font-medium' 
                            : 'bg-[#101010]/50 border-neutral-800/40 text-text-secondary hover:text-text-primary hover:bg-[#151515]'
                        }`}
                      >
                        <span>{v.name}</span>
                        <div className={`w-4 h-4 rounded-full border flex items-center justify-center shrink-0 ${
                          isSelected ? 'border-blue-400' : 'border-neutral-700'
                        }`}>
                          {isSelected && <div className="w-2 h-2 rounded-full bg-blue-400" />}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Option B: Custom Signature Text */}
              <div className="space-y-2.5">
                <label className="block text-[10px] font-mono text-text-secondary/60 uppercase tracking-wider">
                  Email Signature
                </label>
                <div className="relative">
                  <input
                    type="text"
                    value={mailSignature}
                    onChange={(e) => {
                      setMailSignature(e.target.value);
                      localStorage.setItem('kestrel_mail_signature', e.target.value);
                    }}
                    placeholder="e.g. Sent from Kestrel Mail"
                    className="w-full bg-[#101010] border border-neutral-800 hover:border-neutral-700/80 rounded-xl px-3.5 py-2.5 text-xs text-text-primary outline-none focus:border-blue-500/50 transition-all font-sans"
                  />
                </div>
                <p className="text-[10px] font-mono text-text-secondary/40 leading-relaxed">
                  Configures the signature appended to composed emails.
                </p>
              </div>

              {/* Option C: Compact Layout Density Toggle */}
              <div className="flex items-center justify-between p-3 rounded-xl bg-[#101010]/30 border border-neutral-800/40">
                <div className="space-y-1">
                  <span className="block text-xs font-semibold text-text-primary">
                    Compact Row Density
                  </span>
                  <span className="block text-[10px] font-mono text-text-secondary/40">
                    Use high-density rows for email lists.
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => {
                    const val = !mailDenseMode;
                    setMailDenseMode(val);
                    localStorage.setItem('kestrel_mail_dense_mode', val.toString());
                  }}
                  className="text-text-primary hover:text-white transition-colors cursor-pointer"
                >
                  {mailDenseMode ? (
                    <ToggleRight className="w-8 h-8 text-blue-400" />
                  ) : (
                    <ToggleLeft className="w-8 h-8 text-text-secondary/40" />
                  )}
                </button>
              </div>

            </div>

            {/* Modal Footer */}
            <div className="px-5 py-3 border-t border-neutral-800/60 bg-[#181818] flex items-center justify-end">
              <button
                onClick={() => setIsSettingsOpen(false)}
                className="w-full py-2 rounded-xl bg-white hover:bg-neutral-200 text-black text-xs font-semibold shadow transition-all cursor-pointer"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Reusable Custom Label Context Menu */}
      {contextMenu && (
        <>
          <div 
            id="label-context-backdrop"
            className="fixed inset-0 z-50 cursor-default" 
            onClick={() => setContextMenu(null)}
            onContextMenu={(e) => {
              e.preventDefault();
              setContextMenu(null);
            }}
          />
          <div 
            id="label-context-menu"
            className="fixed bg-[#1a1919] border border-white/10 rounded-xl shadow-2xl py-1.5 w-52 z-50 font-sans"
            style={{ 
              top: contextMenu.y, 
              left: Math.min(contextMenu.x, window.innerWidth - 220) 
            }}
          >
            {/* Row 1: Icon + Text Input */}
            <div className="flex items-center gap-2 px-3 py-1.5 border-b border-white/5 mb-1.5 bg-canvas-base/30">
              {/* Clickable Icon Button */}
              <button
                type="button"
                onClick={() => {
                  setShowIconsDropdown(!showIconsDropdown);
                  setShowColorsDropdown(false);
                  setShowNestingDropdown(false);
                }}
                title="Change Icon"
                className="p-1 rounded hover:bg-canvas-hover text-text-primary transition-all cursor-pointer flex items-center justify-center shrink-0 border border-white/5 bg-[#1c1b1b]"
              >
                {(() => {
                  const IconComp = iconMapping[editIcon] || Tag;
                  const colorConfig = colorConfigs[editColor];
                  const colorClass = colorConfig ? colorConfig.text : 'text-text-primary';
                  return <IconComp className={`w-3.5 h-3.5 ${colorClass}`} />;
                })()}
              </button>
              
              {/* Name input */}
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                    const parentPath = lastSlashIndex !== -1 ? contextMenu.label.substring(0, lastSlashIndex + 1) : '';
                    const newFullName = parentPath + editName.trim();
                    if (editName.trim() && newFullName !== contextMenu.label) {
                      onRenameLabel(contextMenu.label, newFullName);
                      onUpdateLabelCustomization(newFullName, editIcon, editColor);
                    } else {
                      onUpdateLabelCustomization(contextMenu.label, editIcon, editColor);
                    }
                    setContextMenu(null);
                  }
                }}
                className="w-full bg-transparent border-none text-xs text-text-primary outline-none focus:ring-0 font-sans px-0.5 py-0.5"
                placeholder="Label name"
                autoFocus
              />

              {/* Save checkmark if changed */}
              {(() => {
                const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                const parentPath = lastSlashIndex !== -1 ? contextMenu.label.substring(0, lastSlashIndex + 1) : '';
                const newFullName = parentPath + editName.trim();
                const isChanged = editName.trim() !== '' && newFullName !== contextMenu.label;
                
                return isChanged ? (
                  <button
                    type="button"
                    onClick={() => {
                      onRenameLabel(contextMenu.label, newFullName);
                      onUpdateLabelCustomization(newFullName, editIcon, editColor);
                      setContextMenu(null);
                    }}
                    className="p-1 rounded hover:bg-canvas-hover text-green-400 transition-colors cursor-pointer shrink-0"
                  >
                    <Check className="w-3.5 h-3.5" />
                  </button>
                ) : null;
              })()}
            </div>

            {/* Expandable Icon Selection dropdown */}
            {showIconsDropdown && (
              <div className="px-3 py-1.5 border-b border-white/5 mb-1.5 bg-[#161515]">
                <div className="grid grid-cols-5 gap-1">
                  {Object.keys(iconMapping).map(iconName => {
                    const IconComp = iconMapping[iconName];
                    const isSelected = editIcon === iconName;
                    return (
                      <button
                        key={iconName}
                        type="button"
                        onClick={() => {
                          setEditIcon(iconName);
                          onUpdateLabelCustomization(contextMenu.label, iconName, editColor);
                          setShowIconsDropdown(false);
                        }}
                        title={iconName}
                        className={`p-1 rounded flex items-center justify-center border transition-all cursor-pointer ${
                          isSelected 
                            ? 'bg-blue-500/10 border-blue-500 text-blue-400 font-semibold' 
                            : 'border-white/5 bg-[#1c1b1b] hover:border-white/10 text-text-secondary hover:text-text-primary'
                        }`}
                      >
                        <IconComp className="w-3.5 h-3.5" />
                      </button>
                    );
                  })}
                </div>
              </div>
            )}

            {/* Row 2: Color selector */}
            <div className="relative">
              <button
                type="button"
                onClick={() => {
                  setShowColorsDropdown(!showColorsDropdown);
                  setShowIconsDropdown(false);
                  setShowNestingDropdown(false);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center justify-between cursor-pointer transition-colors"
              >
                <div className="flex items-center gap-2">
                  <span className={`w-3 h-3 rounded-full ${colorConfigs[editColor]?.dot || 'bg-blue-500'}`} />
                  <span>Color</span>
                </div>
                <ChevronRight className={`w-3.5 h-3.5 text-text-secondary transition-transform ${showColorsDropdown ? 'rotate-90' : ''}`} />
              </button>

              {/* Color dots grid - shown beautifully inside the menu when expanded */}
              {showColorsDropdown && (
                <div className="px-3 py-2 bg-[#161515] border-t border-b border-white/5 flex flex-wrap gap-1.5">
                  {Object.keys(colorConfigs).map(colorName => {
                    const colorConfig = colorConfigs[colorName];
                    const isSelected = editColor === colorName;
                    return (
                      <button
                        key={colorName}
                        type="button"
                        onClick={() => {
                          setEditColor(colorName);
                          onUpdateLabelCustomization(contextMenu.label, editIcon, colorName);
                          setShowColorsDropdown(false);
                        }}
                        className={`w-4 h-4 rounded-full ${colorConfig.dot} relative transition-transform cursor-pointer ${
                          isSelected ? 'scale-110 shadow-[0_0_6px_rgba(255,255,255,0.4)] ring-1 ring-white/30' : 'opacity-70 hover:opacity-100 hover:scale-105'
                        }`}
                        title={colorName}
                      >
                        {isSelected && (
                          <span className="absolute inset-0 flex items-center justify-center">
                            <div className="w-1 h-1 rounded-full bg-white/75" />
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Row 3: Nesting selector */}
            <div className="relative">
              <button
                type="button"
                onClick={() => {
                  setShowNestingDropdown(!showNestingDropdown);
                  setShowColorsDropdown(false);
                  setShowIconsDropdown(false);
                }}
                className="w-full px-3 py-2 text-xs text-text-primary hover:bg-canvas-hover flex items-center justify-between cursor-pointer transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Folder className="w-3.5 h-3.5 text-teal-400" />
                  <span>Nest under</span>
                </div>
                <div className="flex items-center gap-1 max-w-[100px] overflow-hidden shrink-0">
                  <span className="text-[10px] text-text-secondary truncate max-w-[60px]">
                    {(() => {
                      const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                      return lastSlashIndex !== -1 ? contextMenu.label.substring(0, lastSlashIndex) : 'None';
                    })()}
                  </span>
                  <ChevronRight className={`w-3.5 h-3.5 text-text-secondary transition-transform ${showNestingDropdown ? 'rotate-90' : ''}`} />
                </div>
              </button>

              {/* Nesting options - shown inside when expanded */}
              {showNestingDropdown && (
                <div className="max-h-[140px] overflow-y-auto bg-[#161515] border-t border-b border-white/5 py-1">
                  {/* None option (top-level) */}
                  <button
                    type="button"
                    onClick={() => {
                      const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                      const segmentName = lastSlashIndex !== -1 ? contextMenu.label.substring(lastSlashIndex + 1) : contextMenu.label;
                      
                      if (contextMenu.label !== segmentName) {
                        onRenameLabel(contextMenu.label, segmentName);
                        onUpdateLabelCustomization(segmentName, editIcon, editColor);
                      }
                      setShowNestingDropdown(false);
                      setContextMenu(null);
                    }}
                    className={`w-full px-4 py-1.5 text-left text-[11px] transition-colors flex items-center justify-between cursor-pointer ${
                      contextMenu.label.indexOf('/') === -1 
                        ? 'text-blue-400 bg-blue-500/5 font-medium' 
                        : 'text-text-secondary hover:text-text-primary hover:bg-canvas-hover/50'
                    }`}
                  >
                    <span>None (Top level)</span>
                    {contextMenu.label.indexOf('/') === -1 && <Check className="w-3 h-3 text-blue-400" />}
                  </button>

                  {/* List of other labels as parents */}
                  {allLabels
                    .filter(l => {
                      // Can't nest under itself or any of its descendants
                      return l !== contextMenu.label && !l.startsWith(contextMenu.label + '/');
                    })
                    .map(parentLabel => {
                      const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                      const currentParent = lastSlashIndex !== -1 ? contextMenu.label.substring(0, lastSlashIndex) : '';
                      const isSelected = currentParent === parentLabel;
                      
                      return (
                        <button
                          key={parentLabel}
                          type="button"
                          onClick={() => {
                            const lastSlashIndex = contextMenu.label.lastIndexOf('/');
                            const segmentName = lastSlashIndex !== -1 ? contextMenu.label.substring(lastSlashIndex + 1) : contextMenu.label;
                            const newFullName = `${parentLabel}/${segmentName}`;
                            
                            if (contextMenu.label !== newFullName) {
                              onRenameLabel(contextMenu.label, newFullName);
                              onUpdateLabelCustomization(newFullName, editIcon, editColor);
                            }
                            setShowNestingDropdown(false);
                            setContextMenu(null);
                          }}
                          className={`w-full px-4 py-1.5 text-left text-[11px] transition-colors flex items-center justify-between cursor-pointer truncate ${
                            isSelected 
                              ? 'text-blue-400 bg-blue-500/5 font-medium' 
                              : 'text-text-secondary hover:text-text-primary hover:bg-canvas-hover/50'
                          }`}
                        >
                          <span className="truncate">{parentLabel}</span>
                          {isSelected && <Check className="w-3 h-3 text-blue-400" />}
                        </button>
                      );
                    })
                  }
                </div>
              )}
            </div>

            {/* Row 4: Delete Label */}
            <button
              id="btn-context-delete-label"
              onClick={() => {
                const label = contextMenu.label;
                setContextMenu(null);
                setEditingLabel(label);
                setIsDeletingConfirmOpen(true);
              }}
              className="w-full px-3 py-2 text-xs text-red-400 hover:bg-red-500/10 flex items-center gap-2 text-left cursor-pointer transition-colors border-t border-white/5 mt-1"
            >
              <Trash2 className="w-3.5 h-3.5 text-red-400" />
              <span>Delete Label</span>
            </button>
          </div>
        </>
      )}

      {/* Custom Delete Confirmation Modal */}
      {isDeletingConfirmOpen && (
        <div 
          id="delete-confirm-dialog-backdrop" 
          className="fixed inset-0 bg-black/30 flex items-center justify-center z-55 p-4 font-sans"
          onClick={() => setIsDeletingConfirmOpen(false)}
        >
          <div 
            id="delete-confirm-dialog" 
            className="bg-[#1a1919] border border-white/10 w-full max-w-xs rounded-xl shadow-xl p-4"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-xs font-semibold text-text-primary mb-1.5">Delete Label</h3>
            <p className="text-[11px] text-text-secondary mb-3 leading-relaxed">
              Are you sure you want to delete <span className="text-text-primary font-mono bg-canvas-base px-1.5 py-0.5 rounded">"{editingLabel}"</span>? This will remove it from all emails permanently.
            </p>
            <div className="flex justify-end gap-1.5">
              <button
                id="btn-cancel-delete-confirm"
                onClick={() => setIsDeletingConfirmOpen(false)}
                className="px-2.5 py-1 rounded text-[11px] font-medium bg-canvas-base hover:bg-canvas-hover text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
              >
                Cancel
              </button>
              <button
                id="btn-submit-delete-confirm"
                onClick={() => {
                  onDeleteLabel(editingLabel!);
                  setIsDeletingConfirmOpen(false);
                  setEditingLabel(null);
                }}
                className="px-3 py-1 rounded text-[11px] font-medium bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/10 transition-colors cursor-pointer"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
