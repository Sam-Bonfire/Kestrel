import React, { useState } from 'react';
import { 
  Palette, 
  Type, 
  Square, 
  Play, 
  Tag, 
  CheckSquare, 
  Layers, 
  Copy, 
  Check, 
  ArrowLeft, 
  Sparkles, 
  Trash2, 
  Search, 
  Mail, 
  Calendar, 
  Paperclip, 
  ChevronRight, 
  Plus, 
  SlidersHorizontal,
  Folder,
  Eye,
  Settings
} from 'lucide-react';

interface ComponentSpec {
  name: string;
  description: string;
  preview: React.ReactNode;
  tailwindClasses: string;
  rawJsx?: string;
}

export default function DesignSystem({ onBack }: { onBack: () => void }) {
  const [activeTab, setActiveTab] = useState<'colors' | 'typography' | 'borders' | 'buttons' | 'tags' | 'inputs' | 'blueprints'>('colors');
  const [copiedValue, setCopiedValue] = useState<string | null>(null);
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);

  const handleCopy = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    setCopiedValue(text);
    setCopiedLabel(label);
    setTimeout(() => {
      setCopiedValue(null);
      setCopiedLabel(null);
    }, 2000);
  };

  // Color Definitions
  const colors = [
    { name: 'Canvas Base', variable: 'var(--color-canvas-base)', hex: '#0D0D0D', twClass: 'bg-canvas-base', desc: 'The absolute dark backdrop of the application. High contrast off-black.' },
    { name: 'Canvas Card', variable: 'var(--color-canvas-card)', hex: '#131313', twClass: 'bg-canvas-card', desc: 'Used for panels, sidebars, modal surfaces and structured layers.' },
    { name: 'Canvas Hover', variable: 'var(--color-canvas-hover)', hex: '#1C1B1B', twClass: 'bg-canvas-hover', desc: 'Background highlight for interactive items under hover states.' },
    { name: 'Canvas Modal', variable: 'var(--color-canvas-modal)', hex: '#2A2A2A', twClass: 'bg-canvas-modal', desc: 'The deepest dialog base for popups, dropdown content, and overlays.' },
    { name: 'Text Primary', variable: 'var(--color-text-primary)', hex: '#E5E2E1', twClass: 'text-text-primary', desc: 'Elegant warm gray used for headers and primary readable content.' },
    { name: 'Text Secondary', variable: 'var(--color-text-secondary)', hex: '#A0A0A0', twClass: 'text-text-secondary', desc: 'Muted neutral gray for subtext, secondary fields, metadata, and labels.' },
    { name: 'Border Hairline', variable: 'var(--color-border-hairline)', hex: '#353534', twClass: 'border-border-hairline', desc: 'Ultra-thin subtle borders separating UI panes without clutter.' },
    { name: 'Today Badge Red', variable: 'N/A', hex: '#D15B47', twClass: 'bg-[#d15b47]', desc: 'Core brand focus accent. Used for the calendar "today" marker and badges.' },
    { name: 'Starred Amber', variable: 'N/A', hex: '#E5B722', twClass: 'text-[#e5b722]', desc: 'Highlighted actions, saved records, and interactive stars.' }
  ];

  // Typography Specs
  const typographies = [
    { name: 'Display Title 1', font: 'Geist (Sans)', classes: 'font-sans font-bold tracking-tight text-2xl text-text-primary', example: 'Kestrel Mail Client', code: '<h1 className="font-sans font-bold tracking-tight text-2xl text-text-primary">...</h1>' },
    { name: 'Section Header', font: 'Geist (Sans)', classes: 'font-sans font-medium text-xs text-text-secondary uppercase tracking-wider', example: 'WORKSPACE INBOX', code: '<h3 className="font-sans font-medium text-xs text-text-secondary uppercase tracking-wider">...</h3>' },
    { name: 'Email Title Row', font: 'Geist (Sans)', classes: 'font-sans font-semibold text-sm text-text-primary', example: 'Product Update: Release v2.4.0 is now live', code: '<span className="font-sans font-semibold text-sm text-text-primary">...</span>' },
    { name: 'Body Text', font: 'Geist (Sans)', classes: 'font-sans text-xs text-text-secondary leading-relaxed', example: 'This release fixes multiple viewport bugs and implements responsive slide-over panels for mobile devices.', code: '<p className="font-sans text-xs text-text-secondary leading-relaxed">...</p>' },
    { name: 'Monospace Code Label', font: 'JetBrains Mono', classes: 'font-mono text-xs text-text-primary tracking-tight', example: '06:26:38 UTC', code: '<span className="font-mono text-xs text-text-primary tracking-tight">...</span>' },
    { name: 'Mini Monospace Badge', font: 'JetBrains Mono', classes: 'font-mono text-[10px] text-text-secondary/60 uppercase', example: 'GCP-US-EAST1', code: '<span className="font-mono text-[10px] text-text-secondary/60 uppercase">...</span>' }
  ];

  // Borders & Hairlines
  const borderSpecs = [
    { name: 'Hairline Horizontal Divider', description: 'Divider separating rows', element: <div className="border-b border-border-hairline w-full my-2" />, classes: 'border-b border-border-hairline' },
    { name: 'Interactive Card Border', description: 'Surrounds cards with subtle feedback hover', element: <div className="p-4 bg-canvas-card border border-border-hairline rounded-lg hover:border-white/20 transition-colors text-xs font-mono">Hover to see outline transition</div>, classes: 'border border-border-hairline hover:border-white/20 transition-colors' },
    { name: 'Focus Input Outline', description: 'Active state outline highlights', element: <input type="text" placeholder="Active input state..." className="bg-canvas-card text-xs text-text-primary rounded-lg px-3 py-1.5 w-full outline-none border border-border-hairline focus:border-white/20 transition-all" />, classes: 'border border-border-hairline focus:border-white/20 transition-all' }
  ];

  // Buttons definitions (visualized + code)
  const buttons: ComponentSpec[] = [
    {
      name: 'Primary Dark Accent',
      description: 'Solid high-contrast white button with clean black label, used for call-to-actions.',
      preview: (
        <button className="bg-white hover:bg-neutral-200 text-black text-xs font-semibold px-3 py-1.5 rounded-lg shadow-md transition-all active:scale-95 cursor-pointer flex items-center gap-1.5">
          <Plus className="w-3.5 h-3.5 stroke-[2.5]" />
          <span>Create New</span>
        </button>
      ),
      tailwindClasses: 'bg-white hover:bg-neutral-200 text-black text-xs font-semibold px-3 py-1.5 rounded-lg transition-all active:scale-95',
      rawJsx: `<button className="bg-white hover:bg-neutral-200 text-black text-xs font-semibold px-3 py-1.5 rounded-lg shadow-md transition-all active:scale-95 cursor-pointer flex items-center gap-1.5">
  <Plus className="w-3.5 h-3.5 stroke-[2.5]" />
  <span>Create New</span>
</button>`
    },
    {
      name: 'Secondary Interactive Border Card',
      description: 'Transparent base with a hairline outline, transitions beautifully on hover.',
      preview: (
        <button className="px-3 py-1.5 bg-canvas-card hover:bg-canvas-hover border border-border-hairline hover:border-white/10 text-text-primary text-xs rounded-lg flex items-center gap-1.5 transition-all cursor-pointer">
          <Settings className="w-3.5 h-3.5" />
          <span>Configure</span>
        </button>
      ),
      tailwindClasses: 'px-3 py-1.5 bg-canvas-card hover:bg-canvas-hover border border-border-hairline text-text-primary text-xs rounded-lg transition-all',
      rawJsx: `<button className="px-3 py-1.5 bg-canvas-card hover:bg-canvas-hover border border-border-hairline hover:border-white/10 text-text-primary text-xs rounded-lg flex items-center gap-1.5 transition-all cursor-pointer">
  <Settings className="w-3.5 h-3.5" />
  <span>Configure</span>
</button>`
    },
    {
      name: 'Ghost Action Icon',
      description: 'No bounding lines, highlights subtly with a tiny opacity layer when hovered.',
      preview: (
        <button className="p-1.5 rounded-lg hover:bg-white/5 text-text-secondary hover:text-text-primary transition-colors cursor-pointer flex items-center justify-center">
          <Palette className="w-4 h-4" />
        </button>
      ),
      tailwindClasses: 'p-1.5 rounded-lg hover:bg-white/5 text-text-secondary hover:text-text-primary transition-colors',
      rawJsx: `<button className="p-1.5 rounded-lg hover:bg-white/5 text-text-secondary hover:text-text-primary transition-colors cursor-pointer flex items-center justify-center">
  <Palette className="w-4 h-4" />
</button>`
    },
    {
      name: 'Muted Delete Button',
      description: 'Subtle translucent amber/red style for risky/destruction events.',
      preview: (
        <button className="p-1.5 rounded-lg hover:bg-red-950/40 text-red-400 hover:text-red-300 transition-colors cursor-pointer flex items-center justify-center border border-red-900/20">
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      ),
      tailwindClasses: 'p-1.5 rounded-lg hover:bg-red-950/40 text-red-400 hover:text-red-300 transition-colors border border-red-900/20',
      rawJsx: `<button className="p-1.5 rounded-lg hover:bg-red-950/40 text-red-400 hover:text-red-300 transition-colors cursor-pointer flex items-center justify-center border border-red-900/20">
  <Trash2 className="w-3.5 h-3.5" />
</button>`
    }
  ];

  // Custom tags & labels definitions
  const tags = [
    { name: 'DevOps / GitHub Tag', color: 'text-blue-400 bg-blue-950/30 border-blue-900/30', tw: 'px-1.5 py-0.5 rounded text-[10px] font-mono font-medium border text-blue-400 bg-blue-950/30 border-blue-900/30' },
    { name: 'Careers / Recruiter Tag', color: 'text-emerald-400 bg-emerald-950/30 border-emerald-900/30', tw: 'px-1.5 py-0.5 rounded text-[10px] font-mono font-medium border text-emerald-400 bg-emerald-950/30 border-emerald-900/30' },
    { name: 'Finance / Statement Tag', color: 'text-purple-400 bg-purple-950/30 border-purple-900/30', tw: 'px-1.5 py-0.5 rounded text-[10px] font-mono font-medium border text-purple-400 bg-purple-950/30 border-purple-900/30' },
    { name: 'Urgent Alert Tag', color: 'text-[#d15b47] bg-red-950/20 border-red-900/30', tw: 'px-1.5 py-0.5 rounded text-[10px] font-mono font-medium border text-[#d15b47] bg-red-950/20 border-red-900/30' }
  ];

  // Input states & fields
  const inputSpecs: ComponentSpec[] = [
    {
      name: 'Omnipresent Search Box',
      description: 'Search box fitted with relative absolute left search icon and right clear action button.',
      preview: (
        <div className="relative max-w-sm w-full">
          <Search className="w-3.5 h-3.5 text-text-secondary absolute left-2.5 top-2.5" />
          <input
            type="text"
            placeholder="Search resources..."
            defaultValue="github.com"
            className="bg-canvas-card hover:bg-canvas-hover/40 focus:bg-canvas-hover text-text-primary text-xs rounded-lg pl-8 pr-8 py-1.5 w-full outline-none border border-border-hairline focus:border-white/20 transition-all placeholder:text-text-secondary/40"
          />
        </div>
      ),
      tailwindClasses: 'bg-canvas-card hover:bg-canvas-hover/40 focus:bg-canvas-hover text-text-primary text-xs rounded-lg border border-border-hairline focus:border-white/20 transition-all pl-8 pr-8 py-1.5 w-full',
      rawJsx: `<div className="relative max-w-sm w-full">
  <Search className="w-3.5 h-3.5 text-text-secondary absolute left-2.5 top-2.5" />
  <input
    type="text"
    placeholder="Search resources..."
    className="bg-canvas-card hover:bg-canvas-hover/40 focus:bg-canvas-hover text-text-primary text-xs rounded-lg pl-8 pr-8 py-1.5 w-full outline-none border border-border-hairline focus:border-white/20 transition-all placeholder:text-text-secondary/40"
  />
</div>`
    },
    {
      name: 'Interactive Checkbox',
      description: 'A custom styled monospace checkbox row design used for selection.',
      preview: (
        <div className="flex items-center gap-2 select-none cursor-pointer p-2 rounded hover:bg-canvas-hover max-w-xs">
          <div className="w-3.5 h-3.5 rounded border border-border-hairline flex items-center justify-center bg-canvas-card text-white hover:border-white/40">
            <Check className="w-2.5 h-2.5 stroke-[3]" />
          </div>
          <span className="text-xs font-mono text-text-primary">Selected State</span>
        </div>
      ),
      tailwindClasses: 'w-3.5 h-3.5 rounded border border-border-hairline bg-canvas-card text-white flex items-center justify-center',
      rawJsx: `<div className="flex items-center gap-2 select-none cursor-pointer">
  <div className="w-3.5 h-3.5 rounded border border-border-hairline flex items-center justify-center bg-canvas-card text-white hover:border-white/40">
    <Check className="w-2.5 h-2.5 stroke-[3]" />
  </div>
  <span className="text-xs font-mono text-text-primary">Selected State</span>
</div>`
    }
  ];

  // Realistic UI Blueprints / Compound Previews
  const blueprints = [
    {
      name: 'Mail Row Blueprint',
      description: 'High-fidelity mockup of the core email row showing unread marker, attachments, stars, and tags.',
      element: (
        <div className="border border-border-hairline rounded-lg bg-canvas-card overflow-hidden w-full max-w-xl">
          <div className="relative flex items-center gap-3 px-3 py-2.5 border-b border-border-hairline/60 bg-canvas-card hover:bg-canvas-hover transition-colors">
            {/* Unread dot indicator */}
            <div className="w-1.5 h-1.5 rounded-full bg-[#d15b47] shrink-0" />
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-semibold text-text-primary truncate">GitHub Notifications</span>
                <span className="text-[10px] font-mono text-text-secondary shrink-0">12:35 PM</span>
              </div>
              <div className="text-xs font-medium text-text-primary truncate mt-0.5">
                [GitHub] Security Alert: 3 vulnerabilities discovered in node_modules
              </div>
              <p className="text-[11px] text-text-secondary truncate mt-0.5">
                We found potential security flaws in your open source package configurations. Please patch to lock dependency issues.
              </p>
              
              <div className="flex items-center gap-1.5 mt-2">
                <span className="px-1.5 py-0.5 rounded text-[9px] font-mono font-medium border text-blue-400 bg-blue-950/20 border-blue-900/20">
                  DevOps
                </span>
                <span className="px-1.5 py-0.5 rounded text-[9px] font-mono font-medium border text-[#d15b47] bg-red-950/10 border-red-900/20">
                  Security
                </span>
                <div className="flex items-center text-text-secondary ml-auto gap-2">
                  <Paperclip className="w-3 h-3 text-text-secondary/40" />
                  <span className="text-[10px] font-mono text-text-secondary/30">18.5 KB</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      ),
      code: `<div className="relative flex items-center gap-3 px-3 py-2.5 border-b border-border-hairline bg-canvas-card hover:bg-canvas-hover transition-colors">
  <div className="w-1.5 h-1.5 rounded-full bg-[#d15b47] shrink-0" />
  <div className="flex-1 min-w-0">
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs font-semibold text-text-primary truncate">GitHub Notifications</span>
      <span className="text-[10px] font-mono text-text-secondary shrink-0">12:35 PM</span>
    </div>
    <div className="text-xs font-medium text-text-primary truncate mt-0.5">Security Alert</div>
    <div className="flex items-center gap-1.5 mt-2">
      <span className="px-1.5 py-0.5 rounded text-[9px] font-mono font-medium border text-blue-400 bg-blue-950/20 border-blue-900/20">DevOps</span>
    </div>
  </div>
</div>`
    },
    {
      name: 'Sidebar Item Blueprint',
      description: 'A responsive sidebar navigational row showing folder counts, icon positioning, and select indicators.',
      element: (
        <div className="bg-canvas-card border border-border-hairline rounded-lg p-3 w-full max-w-xs space-y-1">
          {/* Selected Navigation Item */}
          <div className="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-canvas-hover border border-white/5 text-text-primary text-xs font-medium cursor-pointer">
            <div className="flex items-center gap-2.5">
              <Mail className="w-3.5 h-3.5 text-text-primary" />
              <span>Inbox</span>
            </div>
            <span className="text-[10px] font-mono bg-white text-black font-bold px-1.5 py-0.5 rounded">12</span>
          </div>

          {/* Regular Navigation Item */}
          <div className="flex items-center justify-between px-2.5 py-1.5 rounded-md text-text-secondary hover:text-text-primary hover:bg-white/5 text-xs font-medium cursor-pointer transition-colors">
            <div className="flex items-center gap-2.5">
              <Calendar className="w-3.5 h-3.5 text-text-secondary" />
              <span>Calendar</span>
            </div>
            <span className="text-[10px] font-mono text-text-secondary/60 px-1">2</span>
          </div>
        </div>
      ),
      code: `<div className="flex items-center justify-between px-2.5 py-1.5 rounded-md bg-canvas-hover border border-white/5 text-text-primary text-xs font-medium">
  <div className="flex items-center gap-2.5">
    <Mail className="w-3.5 h-3.5 text-text-primary" />
    <span>Inbox</span>
  </div>
  <span className="text-[10px] font-mono bg-white text-black font-bold px-1.5 py-0.5 rounded">12</span>
</div>`
    },
    {
      name: 'Calendar Grid Cell Blueprint',
      description: 'An isolated grid cell illustrating a calendar month view, active selected states, and dots representing multiple events.',
      element: (
        <div className="bg-canvas-card border border-border-hairline rounded-lg p-3 w-32 h-32 flex flex-col justify-between hover:bg-canvas-hover transition-colors">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold text-white bg-[#d15b47] w-5 h-5 rounded-full flex items-center justify-center">
              19
            </span>
            <span className="text-[10px] font-mono text-text-secondary/40">Today</span>
          </div>
          
          <div className="space-y-1">
            <div className="text-[9px] bg-blue-950/40 text-blue-300 border border-blue-900/30 rounded px-1 py-0.5 truncate font-sans">
              Standup Sync
            </div>
            <div className="text-[9px] bg-purple-950/40 text-purple-300 border border-purple-900/30 rounded px-1 py-0.5 truncate font-sans">
              Deploy v2.4
            </div>
          </div>
        </div>
      ),
      code: `<div className="bg-canvas-card border border-border-hairline rounded-lg p-3 w-32 h-32 flex flex-col justify-between">
  <div className="flex items-center justify-between">
    <span className="text-[11px] font-mono font-bold text-white bg-[#d15b47] w-5 h-5 rounded-full flex items-center justify-center">19</span>
  </div>
  <div className="space-y-1">
    <div className="text-[9px] bg-blue-950/40 text-blue-300 border border-blue-900/30 rounded px-1 py-0.5 truncate font-sans">Sync</div>
  </div>
</div>`
    }
  ];

  return (
    <div id="design-system-panel" className="flex-1 bg-canvas-base text-text-primary flex flex-col h-screen overflow-hidden relative select-none">
      
      {/* Top sticky header bar */}
      <header className="px-6 py-4 border-b border-border-hairline flex items-center justify-between bg-[#090909]/95 backdrop-blur-md sticky top-0 z-20">
        <div className="flex items-center gap-3">
          <button 
            onClick={onBack}
            className="p-1.5 rounded-lg hover:bg-white/5 text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-border-hairline/65 flex items-center justify-center"
            title="Return to Inbox"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div>
            <h1 className="text-sm font-bold font-mono tracking-wider text-text-primary uppercase flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-[#e5b722]" />
              Kestrel Design System
            </h1>
            <p className="text-[10px] font-mono text-text-secondary mt-0.5">Hidden specifications panel & interactive component playbook</p>
          </div>
        </div>
        
        {/* Copy Success Banner */}
        {copiedValue && (
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white text-black text-[11px] font-mono px-3 py-1.5 rounded-full shadow-2xl flex items-center gap-1.5 border border-white/20 animate-scale-in">
            <Check className="w-3.5 h-3.5 stroke-[2.5]" />
            <span>Copied {copiedLabel}!</span>
          </div>
        )}

        <div className="flex items-center gap-2 text-xs font-mono text-text-secondary">
          <span className="w-2 h-2 rounded-full bg-[#d15b47] animate-pulse" />
          <span>Active Environment</span>
        </div>
      </header>

      {/* Main split dashboard layout */}
      <div className="flex-1 flex overflow-hidden">
        
        {/* Left Menu Selection Rails */}
        <nav className="w-56 border-r border-border-hairline bg-[#090909] p-4 flex flex-col gap-1 overflow-y-auto shrink-0">
          <div className="text-[10px] font-mono text-text-secondary/40 font-semibold px-2.5 py-1 tracking-wider uppercase mb-2">Specifications</div>
          
          <button
            onClick={() => setActiveTab('colors')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'colors' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Palette className="w-3.5 h-3.5" />
            <span>Colors & Palette</span>
          </button>

          <button
            onClick={() => setActiveTab('typography')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'typography' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Type className="w-3.5 h-3.5" />
            <span>Typography System</span>
          </button>

          <button
            onClick={() => setActiveTab('borders')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'borders' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Square className="w-3.5 h-3.5" />
            <span>Borders & Hairlines</span>
          </button>

          <div className="text-[10px] font-mono text-text-secondary/40 font-semibold px-2.5 py-1 tracking-wider uppercase mt-4 mb-2">Components</div>

          <button
            onClick={() => setActiveTab('buttons')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'buttons' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Play className="w-3.5 h-3.5" />
            <span>Interactive Buttons</span>
          </button>

          <button
            onClick={() => setActiveTab('tags')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'tags' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Tag className="w-3.5 h-3.5" />
            <span>Badges & Category Tags</span>
          </button>

          <button
            onClick={() => setActiveTab('inputs')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'inputs' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <CheckSquare className="w-3.5 h-3.5" />
            <span>Forms & Inputs</span>
          </button>

          <button
            onClick={() => setActiveTab('blueprints')}
            className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer ${
              activeTab === 'blueprints' ? 'bg-canvas-hover border border-white/5 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-white/5 border border-transparent'
            }`}
          >
            <Layers className="w-3.5 h-3.5" />
            <span>Compound Blueprints</span>
          </button>
        </nav>

        {/* Content Preview Stage */}
        <main className="flex-1 bg-canvas-base overflow-y-auto p-8 scrollbar-none">
          <div className="max-w-4xl mx-auto space-y-8 animate-fade-in">
            
            {/* Header Description of tab */}
            <div>
              <span className="text-[10px] font-mono text-[#e5b722] font-semibold tracking-wider uppercase bg-[#e5b722]/5 px-2 py-0.5 rounded border border-[#e5b722]/15">
                Active Section Specification
              </span>
              <h2 className="text-xl font-bold tracking-tight text-text-primary mt-2 uppercase font-sans">
                {activeTab === 'colors' && 'Color Palette & Variables'}
                {activeTab === 'typography' && 'Typography Pairings'}
                {activeTab === 'borders' && 'Structural Borders & Shadows'}
                {activeTab === 'buttons' && 'Component: Interactive Buttons'}
                {activeTab === 'tags' && 'Component: Badges & Tags'}
                {activeTab === 'inputs' && 'Component: Form Controls'}
                {activeTab === 'blueprints' && 'System Blueprints & Micro-layouts'}
              </h2>
              <p className="text-xs text-text-secondary mt-1">
                {activeTab === 'colors' && 'The absolute source of truth for contrast ratios, container backdrops, text layers, and brand accents.'}
                {activeTab === 'typography' && 'Geist and JetBrains Mono scale specifications. Click any specification card to copy the corresponding CSS tailwind class declarations.'}
                {activeTab === 'borders' && 'Hairlines, card outline styles, focus borders, and backdrop overlays used to separate panes cleanly.'}
                {activeTab === 'buttons' && 'A catalog of pure, modular button components built for rapid visual iteration. Click code snippet boxes to copy source code.'}
                {activeTab === 'tags' && 'Design tokens for categories, custom user labels, and dynamic states with background transparencies.'}
                {activeTab === 'inputs' && 'Text boxes, select widgets, checkboxes, and search queries supporting interactive focus borders.'}
                {activeTab === 'blueprints' && 'Advanced assemblies mapping out how lists, items, cards, and grid cells are wired into production components.'}
              </p>
            </div>

            <div className="border-b border-border-hairline/60 w-full" />

            {/* TAB VIEWPORTS */}

            {/* A. Colors Spec */}
            {activeTab === 'colors' && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {colors.map((color, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-4 flex flex-col justify-between hover:border-white/10 transition-colors">
                    <div className="flex items-center gap-3">
                      <div className={`w-12 h-12 rounded-lg shrink-0 shadow-lg border border-white/5 ${color.twClass}`} style={{ backgroundColor: color.hex }} />
                      <div className="min-w-0">
                        <div className="text-xs font-semibold text-text-primary">{color.name}</div>
                        <div className="text-[10px] font-mono text-text-secondary/70 mt-0.5">{color.hex}</div>
                      </div>
                    </div>
                    
                    <p className="text-[11px] text-text-secondary mt-3 leading-relaxed">
                      {color.desc}
                    </p>

                    <div className="mt-4 pt-3 border-t border-border-hairline/60 flex items-center justify-between">
                      <span className="text-[10px] font-mono text-text-secondary/50 truncate max-w-[180px]">{color.twClass}</span>
                      <div className="flex items-center gap-1.5 shrink-0">
                        <button
                          onClick={() => handleCopy(color.hex, `${color.name} HEX`)}
                          className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                          title="Copy Hex Code"
                        >
                          <Copy className="w-3 h-3" />
                        </button>
                        <button
                          onClick={() => handleCopy(color.twClass, `${color.name} Tailwind Class`)}
                          className="px-1.5 py-0.5 rounded bg-canvas-hover text-[9px] font-mono text-text-secondary hover:text-text-primary hover:border-white/10 border border-transparent cursor-pointer transition-all"
                          title="Copy Tailwind class name"
                        >
                          Class
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* B. Typography Spec */}
            {activeTab === 'typography' && (
              <div className="space-y-4">
                {typographies.map((type, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-5 hover:border-white/10 transition-colors">
                    <div className="flex items-center justify-between border-b border-border-hairline/60 pb-2.5 mb-3.5">
                      <div>
                        <span className="text-[10px] font-mono text-text-secondary/50 uppercase">{type.font}</span>
                        <h4 className="text-xs font-semibold text-text-primary mt-0.5">{type.name}</h4>
                      </div>
                      <button
                        onClick={() => handleCopy(type.classes, `${type.name} Classes`)}
                        className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                        title="Copy Tailwind utility classes"
                      >
                        <Copy className="w-3 h-3" />
                      </button>
                    </div>

                    <div className="py-2 overflow-x-auto">
                      <p className={type.classes}>{type.example}</p>
                    </div>

                    <div 
                      onClick={() => handleCopy(type.code, `${type.name} JSX snippet`)}
                      className="mt-4 bg-[#090909] border border-border-hairline rounded-lg p-2.5 font-mono text-[10px] text-text-secondary/60 cursor-pointer hover:bg-canvas-hover hover:text-text-primary transition-all overflow-x-auto"
                      title="Click to copy JSX code block"
                    >
                      {type.code}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* C. Borders & Hairlines Spec */}
            {activeTab === 'borders' && (
              <div className="space-y-5">
                {borderSpecs.map((spec, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-5 hover:border-white/10 transition-colors space-y-4">
                    <div>
                      <h4 className="text-xs font-semibold text-text-primary">{spec.name}</h4>
                      <p className="text-[11px] text-text-secondary mt-0.5">{spec.description}</p>
                    </div>

                    <div className="bg-canvas-base border border-border-hairline/40 rounded-lg p-4 flex items-center justify-center min-h-[80px]">
                      {spec.element}
                    </div>

                    <div className="flex items-center justify-between bg-[#090909] border border-border-hairline rounded-lg p-2.5">
                      <code className="font-mono text-[10px] text-text-secondary/70 truncate">{spec.classes}</code>
                      <button
                        onClick={() => handleCopy(spec.classes, `${spec.name} Classes`)}
                        className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors shrink-0"
                        title="Copy classes"
                      >
                        <Copy className="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* D. Buttons Spec */}
            {activeTab === 'buttons' && (
              <div className="space-y-6">
                {buttons.map((btn, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-5 hover:border-white/10 transition-colors">
                    <div className="flex items-center justify-between border-b border-border-hairline/60 pb-3 mb-4">
                      <div>
                        <h4 className="text-xs font-semibold text-text-primary">{btn.name}</h4>
                        <p className="text-[11px] text-text-secondary mt-0.5">{btn.description}</p>
                      </div>
                      <button
                        onClick={() => handleCopy(btn.tailwindClasses, `${btn.name} Classes`)}
                        className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                        title="Copy classes"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    {/* Previews */}
                    <div className="bg-canvas-base border border-border-hairline/40 rounded-lg p-6 flex items-center justify-center">
                      {btn.preview}
                    </div>

                    {/* Code Snippet Box */}
                    {btn.rawJsx && (
                      <div className="mt-4 space-y-1.5">
                        <span className="text-[9px] font-mono text-text-secondary/40 uppercase block">Copyable JSX Block</span>
                        <div 
                          onClick={() => handleCopy(btn.rawJsx!, `${btn.name} JSX`)}
                          className="bg-[#090909] border border-border-hairline rounded-lg p-3 font-mono text-[10px] text-text-secondary/70 whitespace-pre overflow-x-auto cursor-pointer hover:bg-canvas-hover hover:text-text-primary transition-all"
                        >
                          {btn.rawJsx}
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* E. Badges & Tags Spec */}
            {activeTab === 'tags' && (
              <div className="space-y-4">
                <div className="bg-canvas-card border border-border-hairline rounded-xl p-5">
                  <h4 className="text-xs font-semibold text-text-primary mb-3.5">Category Color Chips</h4>
                  <div className="flex flex-wrap gap-2.5">
                    {tags.map((tag, idx) => (
                      <div 
                        key={idx}
                        onClick={() => handleCopy(tag.tw, tag.name)}
                        className={`cursor-pointer transition-all hover:scale-105 active:scale-95 ${tag.tw}`}
                        title="Click to copy CSS class parameters"
                      >
                        {tag.name.replace('Tag', '')}
                      </div>
                    ))}
                  </div>
                  <span className="text-[9px] font-mono text-text-secondary/40 block mt-4">Tip: Click tags to copy classes directly to your clipboard.</span>
                </div>

                <div className="bg-canvas-card border border-border-hairline rounded-xl p-5 space-y-3">
                  <h4 className="text-xs font-semibold text-text-primary">Custom Tag Styles (Editable Labels)</h4>
                  <p className="text-[11px] text-text-secondary">Used within labels panels to construct responsive tags.</p>
                  
                  <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 pt-2">
                    <span className="px-2 py-1 rounded bg-[#ff4a4a]/10 border border-[#ff4a4a]/20 text-[#ff4a4a] text-[10px] font-medium font-mono text-center">Crimson</span>
                    <span className="px-2 py-1 rounded bg-[#4ade80]/10 border border-[#4ade80]/20 text-[#4ade80] text-[10px] font-medium font-mono text-center">Emerald</span>
                    <span className="px-2 py-1 rounded bg-[#60a5fa]/10 border border-[#60a5fa]/20 text-[#60a5fa] text-[10px] font-medium font-mono text-center">Sky Blue</span>
                    <span className="px-2 py-1 rounded bg-[#f59e0b]/10 border border-[#f59e0b]/20 text-[#f59e0b] text-[10px] font-medium font-mono text-center">Amber</span>
                  </div>
                </div>
              </div>
            )}

            {/* F. Forms & Inputs */}
            {activeTab === 'inputs' && (
              <div className="space-y-6">
                {inputSpecs.map((spec, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-5 hover:border-white/10 transition-colors">
                    <div className="flex items-center justify-between border-b border-border-hairline/60 pb-3 mb-4">
                      <div>
                        <h4 className="text-xs font-semibold text-text-primary">{spec.name}</h4>
                        <p className="text-[11px] text-text-secondary mt-0.5">{spec.description}</p>
                      </div>
                      <button
                        onClick={() => handleCopy(spec.tailwindClasses, `${spec.name} Classes`)}
                        className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                        title="Copy classes"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <div className="bg-canvas-base border border-border-hairline/40 rounded-lg p-6 flex items-center justify-center">
                      {spec.preview}
                    </div>

                    {spec.rawJsx && (
                      <div className="mt-4 space-y-1.5">
                        <span className="text-[9px] font-mono text-text-secondary/40 uppercase block">Copyable JSX Block</span>
                        <div 
                          onClick={() => handleCopy(spec.rawJsx!, `${spec.name} JSX`)}
                          className="bg-[#090909] border border-border-hairline rounded-lg p-3 font-mono text-[10px] text-text-secondary/70 whitespace-pre overflow-x-auto cursor-pointer hover:bg-canvas-hover hover:text-text-primary transition-all"
                        >
                          {spec.rawJsx}
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* G. Component Blueprints */}
            {activeTab === 'blueprints' && (
              <div className="space-y-6">
                {blueprints.map((blueprint, idx) => (
                  <div key={idx} className="bg-canvas-card border border-border-hairline rounded-xl p-5 hover:border-white/10 transition-colors">
                    <div className="flex items-center justify-between border-b border-border-hairline/60 pb-3 mb-4">
                      <div>
                        <h4 className="text-xs font-semibold text-text-primary">{blueprint.name}</h4>
                        <p className="text-[11px] text-text-secondary mt-0.5">{blueprint.description}</p>
                      </div>
                      <button
                        onClick={() => handleCopy(blueprint.code, `${blueprint.name} Code`)}
                        className="p-1 rounded hover:bg-canvas-hover text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
                        title="Copy full component markup"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <div className="bg-canvas-base border border-border-hairline/40 rounded-lg p-6 flex items-center justify-center overflow-x-auto">
                      {blueprint.element}
                    </div>

                    <div className="mt-4">
                      <span className="text-[9px] font-mono text-text-secondary/40 uppercase block mb-1.5">Full HTML / Component Template</span>
                      <div 
                        onClick={() => handleCopy(blueprint.code, `${blueprint.name} Code`)}
                        className="bg-[#090909] border border-border-hairline rounded-lg p-3 font-mono text-[10px] text-text-secondary/70 whitespace-pre overflow-x-auto cursor-pointer hover:bg-canvas-hover hover:text-text-primary transition-all"
                      >
                        {blueprint.code}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

          </div>
        </main>
      </div>

    </div>
  );
}
