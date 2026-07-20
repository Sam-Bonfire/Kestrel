/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

export interface Email {
  id: string;
  sender: string;
  senderEmail: string;
  to: string;
  subject: string;
  body: string; // HTML or Markdown string for the email content
  timestamp: string; // ISO date string
  isUnread: boolean;
  isArchived: boolean;
  isStarred: boolean;
  isDraft: boolean;
  isSpam: boolean;
  isTrash: boolean;
  hasAttachment: boolean;
  category: string; // e.g. "Primary", "Updates", "Social", "Promotions", "Forums"
  labels: string[]; // custom tags/labels (e.g., ["Updates [Gmail]", "Work", "Urgent"])
  avatar?: string; // Optional sender avatar URL or initials
}

export type ViewType = 
  | 'inbox' 
  | 'unread'
  | 'all-mail'
  | 'sent'
  | 'drafts'
  | 'spam'
  | 'trash'
  | 'starred'
  | 'label' // dynamically filter by specific label
  | 'github' // custom view matching the image
  | 'categories'; // dynamically filter by Category [Gmail]

export interface FilterState {
  category: string | null;
  label: string | null;
  isUnread: boolean;
  showArchived: boolean;
  customChips: Array<{ id: string; field: string; value: string }>;
}
