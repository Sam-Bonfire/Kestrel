import React, { useState } from 'react';
import { X, Send, Paperclip } from 'lucide-react';
import { Email } from '../types';

interface ComposeModalProps {
  onClose: () => void;
  onSend: (newEmail: Omit<Email, 'id' | 'timestamp'>) => void;
}

export default function ComposeModal({ onClose, onSend }: ComposeModalProps) {
  const [to, setTo] = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState(() => {
    try {
      const sig = localStorage.getItem('kestrel_mail_signature') || 'Sent from Kestrel Mail';
      return `\n\n--\n${sig}`;
    } catch {
      return '\n\n--\nSent from Kestrel Mail';
    }
  });
  const [hasAttachment, setHasAttachment] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!to.trim() || !subject.trim() || !body.trim()) {
      alert("Please fill in all the fields before sending.");
      return;
    }

    onSend({
      sender: 'Developer',
      senderEmail: 'user@workspace.io',
      to: to.trim(),
      subject: subject.trim(),
      body: `<p style="font-family: sans-serif; line-height: 1.6; color: #e5e2e1;">${body.replace(/\n/g, '<br/>')}</p>`,
      isUnread: false,
      isArchived: false,
      isStarred: false,
      isDraft: false,
      isSpam: false,
      isTrash: false,
      hasAttachment,
      category: 'Primary',
      labels: ['Sent']
    });

    onClose();
  };

  return (
    <div 
      id="compose-modal-overlay" 
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs select-none"
    >
      <div 
        id="compose-modal-card" 
        className="w-full max-w-xl bg-canvas-card rounded-lg shadow-2xl flex flex-col font-sans"
      >
        {/* Header bar */}
        <div id="compose-header" className="px-4 py-3 bg-[#1c1b1b] flex items-center justify-between mb-1">
          <span className="text-xs font-mono font-medium text-text-primary uppercase tracking-wider">New Message</span>
          <button
            id="btn-close-compose"
            onClick={onClose}
            className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form elements */}
        <form onSubmit={handleSubmit} className="p-4 flex flex-col gap-4">
          
          {/* Recipient line */}
          <div className="flex items-center gap-3 pb-3 mb-1">
            <span className="text-xs font-mono text-text-secondary w-12 text-right">To:</span>
            <input
              id="compose-to-input"
              type="email"
              required
              placeholder="recipient@example.com"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              className="flex-1 bg-transparent border-none outline-none text-sm text-text-primary placeholder:text-text-secondary/40 font-sans"
            />
          </div>

          {/* Subject line */}
          <div className="flex items-center gap-3 pb-3 mb-1">
            <span className="text-xs font-mono text-text-secondary w-12 text-right">Subject:</span>
            <input
              id="compose-subject-input"
              type="text"
              required
              placeholder="What's this mail about?"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              className="flex-1 bg-transparent border-none outline-none text-sm text-text-primary placeholder:text-text-secondary/40 font-sans font-medium"
            />
          </div>

          {/* Mail body input block */}
          <div className="flex flex-col gap-1">
            <textarea
              id="compose-body-input"
              required
              placeholder="Draft your mail content here..."
              value={body}
              onChange={(e) => setBody(e.target.value)}
              className="w-full h-40 sm:h-60 md:h-80 bg-transparent border-none outline-none resize-none text-sm text-text-primary placeholder:text-text-secondary/35 font-sans leading-relaxed"
            />
          </div>

          {/* Actions panel */}
          <div className="flex items-center justify-between pt-5 mt-4 select-none">
            
            {/* Attachment simulation trigger */}
            <button
              id="compose-attachment-btn"
              type="button"
              onClick={() => setHasAttachment(!hasAttachment)}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded transition-colors ${
                hasAttachment 
                  ? 'bg-canvas-hover text-white font-medium shadow-sm' 
                  : 'bg-canvas-card text-text-secondary hover:text-text-primary'
              }`}
            >
              <Paperclip className="w-3.5 h-3.5" />
              <span>{hasAttachment ? 'StatementAttached.pdf' : 'Add Attachment'}</span>
            </button>

            {/* Action buttons */}
            <div className="flex items-center gap-2">
              <button
                id="compose-btn-cancel"
                type="button"
                onClick={onClose}
                className="px-3.5 py-2 rounded-md text-xs text-text-secondary hover:text-text-primary hover:bg-canvas-hover cursor-pointer bg-canvas-card"
              >
                Cancel
              </button>
              <button
                id="compose-btn-send"
                type="submit"
                className="flex items-center gap-1.5 px-4 py-1.5 bg-white text-canvas-base font-semibold rounded-md text-xs hover:bg-white/90 cursor-pointer"
              >
                <Send className="w-3.5 h-3.5" />
                <span>Send message</span>
              </button>
            </div>

          </div>

        </form>
      </div>
    </div>
  );
}
