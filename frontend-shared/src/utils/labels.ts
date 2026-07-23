export interface LabelMeta {
  name: string;
  iconName: string;
  colorName: string;
  bgColor: string;
  textColor: string;
  borderColor: string;
  dotColor: string;
}

export interface LabelNode {
  name: string;
  displayName: string;
  children: LabelNode[];
  isReal: boolean;
}

export interface FlattenedLabelItem {
  name: string;
  displayName: string;
  depth: number;
  hasChildren: boolean;
  isExpanded: boolean;
  isReal: boolean;
}

// Fallback configs deterministically generated
const FALLBACK_COLORS = [
  { name: 'blue', text: 'text-blue-400', bg: 'bg-blue-500/10 hover:bg-blue-500/20', border: 'border-blue-500/20', dot: 'bg-blue-400' },
  { name: 'emerald', text: 'text-emerald-400', bg: 'bg-emerald-500/10 hover:bg-emerald-500/20', border: 'border-emerald-500/20', dot: 'bg-emerald-400' },
  { name: 'pink', text: 'text-pink-400', bg: 'bg-pink-500/10 hover:bg-pink-500/20', border: 'border-pink-500/20', dot: 'bg-pink-400' },
  { name: 'violet', text: 'text-violet-400', bg: 'bg-violet-500/10 hover:bg-violet-500/20', border: 'border-violet-500/20', dot: 'bg-violet-400' },
  { name: 'amber', text: 'text-amber-400', bg: 'bg-amber-500/10 hover:bg-amber-500/20', border: 'border-amber-500/20', dot: 'bg-amber-400' },
  { name: 'orange', text: 'text-orange-400', bg: 'bg-orange-500/10 hover:bg-orange-500/20', border: 'border-orange-500/20', dot: 'bg-orange-400' },
  { name: 'teal', text: 'text-teal-400', bg: 'bg-teal-500/10 hover:bg-teal-500/20', border: 'border-teal-500/20', dot: 'bg-teal-400' },
  { name: 'red', text: 'text-red-400', bg: 'bg-red-500/10 hover:bg-red-500/20', border: 'border-red-500/20', dot: 'bg-red-400' },
  { name: 'purple', text: 'text-purple-400', bg: 'bg-purple-500/10 hover:bg-purple-500/20', border: 'border-purple-500/20', dot: 'bg-purple-400' },
  { name: 'cyan', text: 'text-cyan-400', bg: 'bg-cyan-500/10 hover:bg-cyan-500/20', border: 'border-cyan-500/20', dot: 'bg-cyan-400' },
  { name: 'rose', text: 'text-rose-400', bg: 'bg-rose-500/10 hover:bg-rose-500/20', border: 'border-rose-500/20', dot: 'bg-rose-400' },
  { name: 'slate', text: 'text-slate-300', bg: 'bg-slate-400/10 hover:bg-slate-400/20', border: 'border-slate-400/20', dot: 'bg-slate-300' }
];

const PRESETS: Record<string, { icon: string; color: string }> = {
  'updates': { icon: 'Info', color: 'blue' },
  'careers': { icon: 'Briefcase', color: 'emerald' },
  'job alerts': { icon: 'Bell', color: 'teal' },
  'indeed matches': { icon: 'Sparkles', color: 'sky' },
  'finance': { icon: 'Coins', color: 'amber' },
  'statements': { icon: 'FileText', color: 'orange' },
  'subscriptions': { icon: 'CreditCard', color: 'violet' },
  'billing': { icon: 'Receipt', color: 'rose' },
  'recruitment': { icon: 'UserCheck', color: 'indigo' },
  'american express': { icon: 'CreditCard', color: 'blue' },
  'referrals': { icon: 'Users', color: 'purple' },
  'devops': { icon: 'Terminal', color: 'slate' },
  'notion mail': { icon: 'BookOpen', color: 'amber' },
  'sent': { icon: 'Send', color: 'violet' },
  'replied': { icon: 'Reply', color: 'lime' },
  'github': { icon: 'GitBranch', color: 'indigo' },
  'work': { icon: 'Briefcase', color: 'cyan' },
  'urgent': { icon: 'AlertCircle', color: 'red' }
};

export function getLabelStyle(
  labelName: string,
  customizations?: Record<string, { iconName: string; colorName: string }>
): LabelMeta {
  const norm = labelName.toLowerCase();
  
  // 1. Check custom overrides
  if (customizations && customizations[labelName]) {
    const cust = customizations[labelName];
    const colMatch = FALLBACK_COLORS.find(c => c.name === cust.colorName) || FALLBACK_COLORS[0];
    return {
      name: labelName,
      iconName: cust.iconName,
      colorName: cust.colorName,
      bgColor: colMatch.bg,
      textColor: colMatch.text,
      borderColor: colMatch.border,
      dotColor: colMatch.dot
    };
  }

  // 2. Check preset mapping
  const preset = PRESETS[norm];
  if (preset) {
    const colMatch = FALLBACK_COLORS.find(c => c.name === preset.color) || FALLBACK_COLORS[0];
    return {
      name: labelName,
      iconName: preset.icon,
      colorName: preset.color,
      bgColor: colMatch.bg,
      textColor: colMatch.text,
      borderColor: colMatch.border,
      dotColor: colMatch.dot
    };
  }

  // 3. Fallback based on hash
  let hash = 0;
  for (let i = 0; i < labelName.length; i++) {
    hash = labelName.charCodeAt(i) + ((hash << 5) - hash);
  }
  const colMatch = FALLBACK_COLORS[Math.abs(hash) % FALLBACK_COLORS.length];
  return {
    name: labelName,
    iconName: 'Tag',
    colorName: colMatch.name,
    bgColor: colMatch.bg,
    textColor: colMatch.text,
    borderColor: colMatch.border,
    dotColor: colMatch.dot
  };
}

export function buildLabelTree(labels: string[]): LabelNode[] {
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
}

export function getFlattenedLabels(
  labels: string[],
  collapsedLabels: Record<string, boolean>
): FlattenedLabelItem[] {
  const tree = buildLabelTree(labels);
  const result: FlattenedLabelItem[] = [];

  const traverse = (nodes: LabelNode[], depth: number) => {
    const sorted = [...nodes].sort((a, b) => a.displayName.localeCompare(b.displayName));
    
    sorted.forEach(node => {
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
}
