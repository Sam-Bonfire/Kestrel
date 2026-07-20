import React, { useState } from 'react';
import { 
  X, 
  ChevronUp, 
  ChevronDown, 
  Sparkles, 
  Star, 
  Clock, 
  Archive, 
  Trash2, 
  Mail, 
  Reply, 
  Forward, 
  Plus, 
  CornerUpLeft, 
  Send,
  MoreVertical,
  Bell
} from 'lucide-react';
import { Email } from '../types';
import { getLabelConfig, iconMapping, colorConfigs } from '../data/labelConfig';

const getInitials = (name: string) => {
  if (!name) return "?";
  const cleanName = name.replace(/[<>"']/g, '').trim();
  const parts = cleanName.split(/\s+/);
  if (parts.length > 1) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }
  return cleanName.substring(0, 2).toUpperCase();
};

const getAvatarColor = (name: string) => {
  const colors = [
    'bg-red-500/10 text-red-400 border-red-500/20',
    'bg-orange-500/10 text-orange-400 border-orange-500/20',
    'bg-amber-500/10 text-amber-400 border-amber-500/20',
    'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    'bg-teal-500/10 text-teal-400 border-teal-500/20',
    'bg-cyan-500/10 text-cyan-400 border-cyan-500/20',
    'bg-sky-500/10 text-sky-400 border-sky-500/20',
    'bg-blue-500/10 text-blue-400 border-blue-500/20',
    'bg-indigo-500/10 text-indigo-400 border-indigo-500/20',
    'bg-violet-500/10 text-violet-400 border-violet-500/20',
    'bg-fuchsia-500/10 text-fuchsia-400 border-fuchsia-500/20',
    'bg-pink-500/10 text-pink-400 border-pink-500/20',
    'bg-rose-500/10 text-rose-400 border-rose-500/20',
  ];
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const index = Math.abs(hash) % colors.length;
  return colors[index];
};

interface CenterPeekProps {
  email: Email | null;
  onClose: () => void;
  onNavigate: (direction: 'prev' | 'next') => void;
  hasPrev: boolean;
  hasNext: boolean;
  onArchive: (id: string) => void;
  onDelete: (id: string) => void;
  onToggleStar: (id: string) => void;
  onToggleUnread: (id: string) => void;
  onAddLabel: (id: string, label: string) => void;
  onRemoveLabel: (id: string, label: string) => void;
  onSendReply: (emailId: string, replyBody: string) => void;
  labelCustomizations?: Record<string, { iconName: string; colorName: string }>;
}

export default function CenterPeek({
  email,
  onClose,
  onNavigate,
  hasPrev,
  hasNext,
  onArchive,
  onDelete,
  onToggleStar,
  onToggleUnread,
  onAddLabel,
  onRemoveLabel,
  onSendReply,
  labelCustomizations
}: CenterPeekProps) {
  const [newLabelText, setNewLabelText] = useState('');
  const [showAddLabelInput, setShowAddLabelInput] = useState(false);
  const [replyText, setReplyText] = useState('');
  const [showReplyDraft, setShowReplyDraft] = useState(false);
  const [mobileMoreMenuOpen, setMobileMoreMenuOpen] = useState(false);

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

  if (!email) return null;

  // Handles adding custom labels
  const handleAddLabelSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newLabelText.trim()) {
      onAddLabel(email.id, newLabelText.trim());
      setNewLabelText('');
      setShowAddLabelInput(false);
    }
  };

  // Handles sending the inline reply draft
  const handleSendReplySubmit = () => {
    if (replyText.trim()) {
      onSendReply(email.id, replyText.trim());
      setReplyText('');
      setShowReplyDraft(false);
    }
  };

  return (
    <div 
      id="center-peek-overlay" 
      className="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4 bg-black/15 backdrop-blur-[1px]"
      onClick={onClose}
    >
      {/* Modal Container */}
      <div 
        id="center-peek-modal" 
        className="w-full md:max-w-4xl h-screen md:h-[85vh] bg-canvas-card flex flex-col rounded-none md:rounded-lg shadow-2xl overflow-hidden font-sans"
        onClick={(e) => e.stopPropagation()}
      >
        
        {/* Header toolbar */}
        <div id="peek-header" className="hidden md:flex px-4 py-3 bg-[#1c1b1b] items-center justify-between select-none">
          
          {/* Left tools: Close and navigation chevrons */}
          <div className="flex items-center gap-1">
            <button
              id="btn-peek-close"
              onClick={onClose}
              title="Close peek (Esc)"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
            <div className="w-2" />
            <button
              id="btn-peek-prev"
              disabled={!hasPrev}
              onClick={() => onNavigate('prev')}
              title="Previous Email"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary disabled:opacity-30 disabled:pointer-events-none cursor-pointer transition-colors"
            >
              <ChevronUp className="w-4 h-4" />
            </button>
            <button
              id="btn-peek-next"
              disabled={!hasNext}
              onClick={() => onNavigate('next')}
              title="Next Email"
              className="p-1.5 rounded-lg hover:bg-canvas-hover text-text-secondary hover:text-text-primary disabled:opacity-30 disabled:pointer-events-none cursor-pointer transition-colors"
            >
              <ChevronDown className="w-4 h-4" />
            </button>
          </div>

          {/* Right tools */}
          <div className="hidden md:flex items-center gap-1.5 text-text-secondary">
            
            <button
              id="btn-peek-star"
              onClick={() => onToggleStar(email.id)}
              title={email.isStarred ? "Remove Star" : "Star Email"}
              className={`p-1.5 rounded-lg hover:bg-canvas-hover cursor-pointer transition-colors ${
                email.isStarred ? 'text-white' : 'hover:text-text-primary'
              }`}
            >
              <Star className={`w-4 h-4 ${email.isStarred ? 'fill-white text-white' : ''}`} />
            </button>

            <button
              id="btn-peek-unread"
              onClick={() => {
                onToggleUnread(email.id);
                onClose();
              }}
              title="Mark as Unread"
              className="p-1.5 rounded-lg hover:bg-canvas-hover hover:text-text-primary cursor-pointer transition-colors"
            >
              <Mail className="w-4 h-4" />
            </button>

            <button
              id="btn-peek-archive"
              onClick={() => {
                onArchive(email.id);
                onClose();
              }}
              title="Archive Email (e)"
              className="p-1.5 rounded-lg hover:bg-canvas-hover hover:text-text-primary cursor-pointer transition-colors"
            >
              <Archive className="w-4 h-4" />
            </button>

            <button
              id="btn-peek-delete"
              onClick={() => {
                onDelete(email.id);
                onClose();
              }}
              title="Delete Email"
              className="p-1.5 rounded-lg hover:bg-canvas-hover hover:text-text-primary cursor-pointer transition-colors"
            >
              <Trash2 className="w-4 h-4" />
            </button>

            <button
              id="btn-peek-more"
              onClick={() => alert("Additional actions: Snooze, Bell, and Mute configurations.")}
              title="More Actions"
              className="p-1.5 rounded-lg hover:bg-canvas-hover hover:text-text-primary cursor-pointer transition-colors"
            >
              <MoreVertical className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Modal Scrollable Content Container */}
        <div id="peek-scrollable-body" className="flex-1 overflow-y-auto px-4 md:px-8 py-6 space-y-6">
          
          {/* Subject Title and Labels Bar */}
          <div className="space-y-1.5">
            <h1 className="text-xl md:text-2xl font-semibold tracking-tight text-text-primary">
              {email.subject}
            </h1>

            {/* Label Management Section */}
            <div id="peek-labels-bar" className="flex flex-wrap items-center gap-1.5">
              {email.labels.map(label => {
                const labelMeta = getLabelStyle(label);
                const LabelIcon = labelMeta.icon;
                return (
                  <div 
                    key={label}
                    id={`peek-tag-${label}`}
                    className={`flex items-center gap-1 px-2 py-0.5 ${labelMeta.bgColor} ${labelMeta.textColor} ${labelMeta.borderColor} border font-mono text-[10px] rounded-full shadow-sm`}
                  >
                    <LabelIcon className="w-3 h-3" />
                    <span>{label}</span>
                    <button
                      id={`btn-remove-tag-${label}`}
                      onClick={() => onRemoveLabel(email.id, label)}
                      className="hover:bg-black/25 p-0.5 rounded-full text-current cursor-pointer transition-colors"
                    >
                      <X className="w-2 h-2" />
                    </button>
                  </div>
                );
              })}

              {showAddLabelInput ? (
                <form onSubmit={handleAddLabelSubmit} className="flex items-center gap-1.5">
                  <input
                    id="tag-input-field"
                    type="text"
                    autoFocus
                    placeholder="Tag name..."
                    value={newLabelText}
                    onChange={(e) => setNewLabelText(e.target.value)}
                    className="bg-canvas-base outline-none rounded-md px-2 py-0.5 text-[10px] text-text-primary font-mono"
                  />
                  <button
                    type="submit"
                    className="px-2 py-0.5 bg-white text-canvas-base text-[10px] font-medium rounded hover:bg-white/90 cursor-pointer"
                  >
                    Add
                  </button>
                  <button
                    type="button"
                    onClick={() => setShowAddLabelInput(false)}
                    className="p-1 text-text-secondary hover:text-text-primary cursor-pointer"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </form>
              ) : (
                <button
                  id="btn-add-tag-trigger"
                  onClick={() => setShowAddLabelInput(true)}
                  title="Add Label"
                  className="flex items-center justify-center p-1 text-text-secondary hover:text-text-primary rounded-full cursor-pointer bg-canvas-hover hover:bg-canvas-hover/80 transition-colors shadow-sm"
                >
                  <Plus className="w-3.5 h-3.5" />
                </button>
              )}
            </div>
          </div>

          {/* Sender & Recipient Details */}
          <div id="sender-details-card" className="flex items-center justify-between bg-canvas-base/60 rounded-lg p-4 mb-2 gap-4">
            <div className="flex items-center gap-3 min-w-0">
              {/* Sender Avatar */}
              {email.avatar && (
                <div className="w-10 h-10 rounded-full overflow-hidden flex items-center justify-center border border-white/10 shrink-0">
                  {email.avatar.startsWith('http') || email.avatar.startsWith('/') ? (
                    <img 
                      src={email.avatar} 
                      alt={email.sender} 
                      className="w-full h-full object-cover"
                      referrerPolicy="no-referrer"
                    />
                  ) : (
                    <div className={`w-full h-full flex items-center justify-center font-semibold text-sm ${getAvatarColor(email.sender)}`}>
                      {email.avatar}
                    </div>
                  )}
                </div>
              )}
              <div className="space-y-0.5 min-w-0">
                <div className="flex flex-wrap items-center gap-1.5 text-sm min-w-0">
                  <span className="font-medium text-text-primary truncate">{email.sender}</span>
                  <span className="text-text-secondary text-xs font-mono truncate">&lt;{email.senderEmail}&gt;</span>
                </div>
                <div className="text-xs text-text-secondary truncate">
                  <span>To: </span>
                  <span className="font-mono">{email.to}</span>
                </div>
              </div>
            </div>

            {/* Timestamps */}
            <div className="text-right shrink-0">
              <span className="text-xs font-mono text-text-secondary">
                {new Date(email.timestamp).toLocaleString([], { 
                  weekday: 'short', 
                  month: 'short', 
                  day: 'numeric', 
                  hour: '2-digit', 
                  minute: '2-digit' 
                })}
              </span>
            </div>
          </div>

          {/* Render Email HTML Body */}
          <div 
            id="email-body-rendered" 
            className="prose prose-invert max-w-none text-sm text-text-primary font-sans leading-relaxed selection:bg-white/20"
            dangerouslySetInnerHTML={{ __html: email.body }}
          />

          {/* Interactive Response Composer Block */}
          {showReplyDraft ? (
            <div id="reply-composer-container" className="bg-canvas-base/80 p-5 mt-8 space-y-4 rounded-lg">
              <div className="flex items-center gap-2 text-xs text-text-secondary font-mono pb-3">
                <CornerUpLeft className="w-3.5 h-3.5" />
                <span>Replying to <strong>{email.senderEmail}</strong></span>
              </div>
              
              <textarea
                id="reply-textarea-field"
                rows={4}
                autoFocus
                value={replyText}
                onChange={(e) => setReplyText(e.target.value)}
                placeholder="Type your response draft here..."
                className="w-full bg-transparent outline-none resize-none text-sm text-text-primary placeholder:text-text-secondary/50 font-sans"
              />

              <div className="flex items-center justify-between pt-4">
                <div className="text-[10px] font-mono text-text-secondary">
                  Press <kbd className="bg-canvas-hover px-1.5 py-0.5 rounded text-[10px]">Send</kbd> to dispatch.
                </div>
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => setShowReplyDraft(false)}
                    className="px-3.5 py-2 rounded-md text-xs text-text-secondary hover:text-text-primary cursor-pointer bg-canvas-card hover:bg-canvas-hover"
                  >
                    Cancel
                  </button>
                  <button
                    type="button"
                    onClick={handleSendReplySubmit}
                    className="flex items-center gap-1.5 px-3 py-1.5 bg-white text-canvas-base font-medium rounded text-xs hover:bg-white/90 cursor-pointer"
                  >
                    <Send className="w-3.5 h-3.5" />
                    <span>Send reply</span>
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="pt-8 flex justify-center">
              <button
                id="reply-composer-trigger-btn"
                onClick={() => setShowReplyDraft(true)}
                className="flex items-center gap-2 px-4 py-2 bg-canvas-card hover:bg-canvas-hover rounded-md text-xs text-text-primary cursor-pointer transition-colors shadow-sm"
              >
                <Reply className="w-3.5 h-3.5" />
                <span>Write a response...</span>
              </button>
            </div>
          )}

        </div>

        {/* Mobile Bottom Action Bar */}
        <div 
          id="peek-mobile-bottom-actions" 
          className="md:hidden border-t border-white/10 bg-[#1c1b1b] px-4 py-3 flex items-center justify-around select-none"
        >
          <button
            id="mobile-action-close"
            onClick={onClose}
            className="p-2.5 rounded-full hover:bg-canvas-hover text-text-secondary hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
          
          {/* More menu trigger for Mobile */}
          <div className="relative">
            <button
              id="mobile-action-more"
              onClick={() => setMobileMoreMenuOpen(!mobileMoreMenuOpen)}
              className={`p-2.5 rounded-full hover:bg-canvas-hover transition-colors ${
                mobileMoreMenuOpen ? 'text-white bg-canvas-hover' : 'text-text-secondary'
              }`}
            >
              <MoreVertical className="w-5 h-5" />
            </button>
            
            {mobileMoreMenuOpen && (
              <div 
                className="absolute bottom-14 left-0 w-48 bg-[#1e1d1d] border border-white/15 rounded-lg py-1 shadow-2xl z-50 animate-fade-in"
                onClick={(e) => e.stopPropagation()}
              >
                <button
                  onClick={() => {
                    onToggleUnread(email.id);
                    onClose();
                    setMobileMoreMenuOpen(false);
                  }}
                  className="w-full text-left px-4 py-2.5 text-xs text-text-primary hover:bg-white/10 flex items-center gap-2"
                >
                  <Mail className="w-4 h-4" />
                  <span>Mark as Unread</span>
                </button>
                <button
                  onClick={() => {
                    setShowAddLabelInput(true);
                    setMobileMoreMenuOpen(false);
                    setTimeout(() => {
                      const el = document.getElementById('tag-input-field');
                      if (el) el.focus();
                    }, 100);
                  }}
                  className="w-full text-left px-4 py-2.5 text-xs text-text-primary hover:bg-white/10 flex items-center gap-2"
                >
                  <Plus className="w-4 h-4" />
                  <span>Add Label</span>
                </button>
                <button
                  onClick={() => {
                    alert(`Email details: Sender is ${email.senderEmail}, Category: ${email.category}`);
                    setMobileMoreMenuOpen(false);
                  }}
                  className="w-full text-left px-4 py-2.5 text-xs text-text-primary hover:bg-white/10 flex items-center gap-2"
                >
                  <Bell className="w-4 h-4" />
                  <span>Show details</span>
                </button>
              </div>
            )}
          </div>

          <button
            id="mobile-action-star"
            onClick={() => onToggleStar(email.id)}
            className={`p-2.5 rounded-full hover:bg-canvas-hover transition-colors ${
              email.isStarred ? 'text-yellow-400' : 'text-text-secondary hover:text-white'
            }`}
          >
            <Star className={`w-5 h-5 ${email.isStarred ? 'fill-current' : ''}`} />
          </button>

          <button
            id="mobile-action-archive"
            onClick={() => {
              onArchive(email.id);
              onClose();
            }}
            className="p-2.5 rounded-full hover:bg-canvas-hover text-text-secondary hover:text-white transition-colors"
          >
            <Archive className="w-5 h-5" />
          </button>

          <button
            id="mobile-action-reply"
            onClick={() => {
              setShowReplyDraft(true);
              setTimeout(() => {
                const el = document.getElementById('reply-editor-box');
                if (el) el.scrollIntoView({ behavior: 'smooth' });
              }, 100);
            }}
            className="p-2.5 rounded-full hover:bg-canvas-hover text-text-secondary hover:text-white transition-colors"
          >
            <Reply className="w-5 h-5" />
          </button>

          <button
            id="mobile-action-delete"
            onClick={() => {
              onDelete(email.id);
              onClose();
            }}
            className="p-2.5 rounded-full hover:bg-red-500/10 text-text-secondary hover:text-red-400 transition-colors"
          >
            <Trash2 className="w-5 h-5" />
          </button>
        </div>

      </div>
    </div>
  );
}
