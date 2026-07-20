import { 
  Tag, 
  Briefcase, 
  Bell, 
  Sparkles, 
  Coins, 
  FileText, 
  CreditCard, 
  Receipt, 
  UserCheck, 
  Users, 
  Terminal, 
  BookOpen, 
  Send, 
  Reply, 
  GitBranch, 
  AlertCircle,
  Inbox,
  Info
} from 'lucide-react';
import React from 'react';

export interface LabelMeta {
  name: string;
  icon: React.ComponentType<any>;
  color: string; // Color name
  bgColor: string;
  textColor: string;
  borderColor: string;
}

const labelMapping: Record<string, { icon: React.ComponentType<any>; color: string; bg: string; text: string; border: string }> = {
  'Updates [Gmail]': {
    icon: Info,
    color: 'blue',
    bg: 'bg-blue-500/10 hover:bg-blue-500/20',
    text: 'text-blue-400',
    border: 'border-blue-500/20'
  },
  'Careers': {
    icon: Briefcase,
    color: 'emerald',
    bg: 'bg-emerald-500/10 hover:bg-emerald-500/20',
    text: 'text-emerald-400',
    border: 'border-emerald-500/20'
  },
  'Job Alerts': {
    icon: Bell,
    color: 'teal',
    bg: 'bg-teal-500/10 hover:bg-teal-500/20',
    text: 'text-teal-400',
    border: 'border-teal-500/20'
  },
  'Indeed Matches': {
    icon: Sparkles,
    color: 'sky',
    bg: 'bg-sky-500/10 hover:bg-sky-500/20',
    text: 'text-sky-400',
    border: 'border-sky-500/20'
  },
  'Finance': {
    icon: Coins,
    color: 'amber',
    bg: 'bg-amber-500/10 hover:bg-amber-500/20',
    text: 'text-amber-400',
    border: 'border-amber-500/20'
  },
  'Statements': {
    icon: FileText,
    color: 'orange',
    bg: 'bg-orange-500/10 hover:bg-orange-500/20',
    text: 'text-orange-400',
    border: 'border-orange-500/20'
  },
  'Subscriptions': {
    icon: CreditCard,
    color: 'violet',
    bg: 'bg-violet-500/10 hover:bg-violet-500/20',
    text: 'text-violet-400',
    border: 'border-violet-500/20'
  },
  'Billing': {
    icon: Receipt,
    color: 'rose',
    bg: 'bg-rose-500/10 hover:bg-rose-500/20',
    text: 'text-rose-400',
    border: 'border-rose-500/20'
  },
  'Recruitment': {
    icon: UserCheck,
    color: 'indigo',
    bg: 'bg-indigo-500/10 hover:bg-indigo-500/20',
    text: 'text-indigo-400',
    border: 'border-indigo-500/20'
  },
  'American Express': {
    icon: CreditCard,
    color: 'blue',
    bg: 'bg-blue-500/10 hover:bg-blue-500/20',
    text: 'text-blue-400',
    border: 'border-blue-500/20'
  },
  'Referrals': {
    icon: Users,
    color: 'purple',
    bg: 'bg-purple-500/10 hover:bg-purple-500/20',
    text: 'text-purple-400',
    border: 'border-purple-500/20'
  },
  'DevOps': {
    icon: Terminal,
    color: 'slate',
    bg: 'bg-slate-400/10 hover:bg-slate-400/20',
    text: 'text-slate-300',
    border: 'border-slate-400/20'
  },
  'Notion Mail': {
    icon: BookOpen,
    color: 'amber',
    bg: 'bg-amber-500/10 hover:bg-amber-500/20',
    text: 'text-amber-400',
    border: 'border-amber-500/20'
  },
  'Sent': {
    icon: Send,
    color: 'violet',
    bg: 'bg-violet-500/10 hover:bg-violet-500/20',
    text: 'text-violet-400',
    border: 'border-violet-500/20'
  },
  'Replied': {
    icon: Reply,
    color: 'lime',
    bg: 'bg-lime-500/10 hover:bg-lime-500/20',
    text: 'text-lime-400',
    border: 'border-lime-500/20'
  },
  'GitHub': {
    icon: GitBranch,
    color: 'indigo',
    bg: 'bg-indigo-500/10 hover:bg-indigo-500/20',
    text: 'text-indigo-400',
    border: 'border-indigo-500/20'
  },
  'Work': {
    icon: Briefcase,
    color: 'cyan',
    bg: 'bg-cyan-500/10 hover:bg-cyan-500/20',
    text: 'text-cyan-400',
    border: 'border-cyan-500/20'
  },
  'Urgent': {
    icon: AlertCircle,
    color: 'red',
    bg: 'bg-red-500/10 hover:bg-red-500/20',
    text: 'text-red-400',
    border: 'border-red-500/20'
  }
};

// Returns a fallback configuration if label is not explicitly configured
export function getLabelConfig(labelName: string): LabelMeta {
  const matched = labelMapping[labelName];
  if (matched) {
    return {
      name: labelName,
      icon: matched.icon,
      color: matched.color,
      bgColor: matched.bg,
      textColor: matched.text,
      borderColor: matched.border
    };
  }

  // Generate deterministic style based on string hash
  let hash = 0;
  for (let i = 0; i < labelName.length; i++) {
    hash = labelName.charCodeAt(i) + ((hash << 5) - hash);
  }
  const colors = [
    { name: 'blue', text: 'text-blue-400', bg: 'bg-blue-500/10 hover:bg-blue-500/20', border: 'border-blue-500/20' },
    { name: 'emerald', text: 'text-emerald-400', bg: 'bg-emerald-500/10 hover:bg-emerald-500/20', border: 'border-emerald-500/20' },
    { name: 'pink', text: 'text-pink-400', bg: 'bg-pink-500/10 hover:bg-pink-500/20', border: 'border-pink-500/20' },
    { name: 'violet', text: 'text-violet-400', bg: 'bg-violet-500/10 hover:bg-violet-500/20', border: 'border-violet-500/20' },
    { name: 'amber', text: 'text-amber-400', bg: 'bg-amber-500/10 hover:bg-amber-500/20', border: 'border-amber-500/20' },
    { name: 'orange', text: 'text-orange-400', bg: 'bg-orange-500/10 hover:bg-orange-500/20', border: 'border-orange-500/20' },
    { name: 'teal', text: 'text-teal-400', bg: 'bg-teal-500/10 hover:bg-teal-500/20', border: 'border-teal-500/20' }
  ];
  const selectedColor = colors[Math.abs(hash) % colors.length];

  return {
    name: labelName,
    icon: Tag,
    color: selectedColor.name,
    bgColor: selectedColor.bg,
    textColor: selectedColor.text,
    borderColor: selectedColor.border
  };
}

export const iconMapping: Record<string, React.ComponentType<any>> = {
  Tag, 
  Briefcase, 
  Bell, 
  Sparkles, 
  Coins, 
  FileText, 
  CreditCard, 
  Receipt, 
  UserCheck, 
  Users, 
  Terminal, 
  BookOpen, 
  Send, 
  Reply, 
  GitBranch, 
  AlertCircle,
  Inbox,
  Info
};

export const colorConfigs: Record<string, { bg: string; text: string; border: string; dot: string }> = {
  blue: { bg: 'bg-blue-500/10 hover:bg-blue-500/20', text: 'text-blue-400', border: 'border-blue-500/20', dot: 'bg-blue-400' },
  emerald: { bg: 'bg-emerald-500/10 hover:bg-emerald-500/20', text: 'text-emerald-400', border: 'border-emerald-500/20', dot: 'bg-emerald-400' },
  pink: { bg: 'bg-pink-500/10 hover:bg-pink-500/20', text: 'text-pink-400', border: 'border-pink-500/20', dot: 'bg-pink-400' },
  violet: { bg: 'bg-violet-500/10 hover:bg-violet-500/20', text: 'text-violet-400', border: 'border-violet-500/20', dot: 'bg-violet-400' },
  amber: { bg: 'bg-amber-500/10 hover:bg-amber-500/20', text: 'text-amber-400', border: 'border-amber-500/20', dot: 'bg-amber-400' },
  orange: { bg: 'bg-orange-500/10 hover:bg-orange-500/20', text: 'text-orange-400', border: 'border-orange-500/20', dot: 'bg-orange-400' },
  teal: { bg: 'bg-teal-500/10 hover:bg-teal-500/20', text: 'text-teal-400', border: 'border-teal-500/20', dot: 'bg-teal-400' },
  red: { bg: 'bg-red-500/10 hover:bg-red-500/20', text: 'text-red-400', border: 'border-red-500/20', dot: 'bg-red-400' },
  purple: { bg: 'bg-purple-500/10 hover:bg-purple-500/20', text: 'text-purple-400', border: 'border-purple-500/20', dot: 'bg-purple-400' },
  cyan: { bg: 'bg-cyan-500/10 hover:bg-cyan-500/20', text: 'text-cyan-400', border: 'border-cyan-500/20', dot: 'bg-cyan-400' },
  rose: { bg: 'bg-rose-500/10 hover:bg-rose-500/20', text: 'text-rose-400', border: 'border-rose-500/20', dot: 'bg-rose-400' },
  slate: { bg: 'bg-slate-400/10 hover:bg-slate-400/20', text: 'text-slate-300', border: 'border-slate-400/20', dot: 'bg-slate-300' }
};
