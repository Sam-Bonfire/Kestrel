<script lang="ts">
  import {
    Inbox,
    CircleDot,
    Send,
    FileText,
    Github,
    Mail,
    AlertTriangle,
    Trash2,
    Settings,
    PenSquare,
    Search,
    Tag,
    ChevronDown,
    ChevronRight,
    Star,
    Check,
    Folder,
    Briefcase,
    Bell,
    Sparkles,
    Coins,
    CreditCard,
    Receipt,
    UserCheck,
    Users,
    Terminal,
    BookOpen,
    Reply,
    GitBranch,
    AlertCircle,
    Info,
    X,
    Plus
  } from 'lucide-svelte';
  import {
    mailDenseMode,
    mailDefaultLandingView,
    mailSignature,
    labelCustomizations,
    getLabelStyle,
    getFlattenedLabels,
    type LabelMeta,
    Dropdown
  } from '@kestrel/shared';
  import { onMount } from 'svelte';
  import { slide, scale, fade } from 'svelte/transition';

  export interface Account {
    id: string;
    name: string;
    email: string;
    color: string;
  }

  let {
    currentView = 'inbox',
    onSelectView = (view: string) => {},
    onComposeClick = () => {},
    searchQuery = $bindable(''),
    accounts = [] as Account[],
    activeAccountId = $bindable(''),
    allLabels = [] as string[],
    onRenameLabel = (oldVal: string, newVal: string) => {},
    onDeleteLabel = (label: string) => {},
    onCreateNewLabel = (label: string) => {},
    inboxCount = 0,
    unreadCount = 0,
    viewCounts = {} as Record<string, number>
  } = $props<{
    currentView?: string;
    onSelectView?: (view: string) => void;
    onComposeClick?: () => void;
    searchQuery?: string;
    accounts?: Account[];
    activeAccountId?: string;
    allLabels?: string[];
    onRenameLabel?: (oldVal: string, newVal: string) => void;
    onDeleteLabel?: (label: string) => void;
    onCreateNewLabel?: (label: string) => void;
    inboxCount?: number;
    unreadCount?: number;
    viewCounts?: Record<string, number>;
  }>();

  // Preset folders
  const folders = [
    { id: 'inbox',    label: 'Inbox',    icon: Inbox,         color: 'text-blue-400'   },
    { id: 'unread',   label: 'Unread',   icon: CircleDot,     color: 'text-emerald-400'},
    { id: 'sent',     label: 'Sent',     icon: Send,          color: 'text-violet-400' },
    { id: 'drafts',   label: 'Drafts',   icon: FileText,      color: 'text-amber-400'  },
    { id: 'starred',  label: 'Starred',  icon: Star,          color: 'text-yellow-400' },
    { id: 'github',   label: 'GitHub',   icon: Github,        color: 'text-indigo-400' },
    { id: 'all-mail', label: 'All Mail', icon: Mail,          color: 'text-pink-400'   },
    { id: 'spam',     label: 'Spam',     icon: AlertTriangle, color: 'text-orange-400' },
    { id: 'trash',    label: 'Trash',    icon: Trash2,        color: 'text-red-400'    },
  ] as const;

  // Icons map for dynamic rendering
  const iconMapping: Record<string, any> = {
    Tag, Briefcase, Bell, Sparkles, Coins, FileText, CreditCard, Receipt,
    UserCheck, Users, Terminal, BookOpen, Send, Reply, GitBranch, AlertCircle,
    Inbox, Info
  };

  // Color config mappings
  const colorConfigs: Record<string, { dot: string; text: string }> = {
    blue: { dot: 'bg-blue-500', text: 'text-blue-400' },
    purple: { dot: 'bg-purple-500', text: 'text-purple-400' },
    green: { dot: 'bg-emerald-500', text: 'text-emerald-400' },
    orange: { dot: 'bg-orange-500', text: 'text-orange-400' },
    rose: { dot: 'bg-rose-500', text: 'text-rose-400' },
    amber: { dot: 'bg-amber-500', text: 'text-amber-400' },
    teal: { dot: 'bg-teal-500', text: 'text-teal-400' }
  };

  let activeAccount = $derived(accounts.find((a: Account) => a.id === activeAccountId));

  let labelsExpanded = $state(true);
  let categoriesExpanded = $state(false);
  let collapsedLabels = $state<Record<string, boolean>>({});

  // Context Menu state
  let contextMenu = $state<{ x: number; y: number; label: string } | null>(null);
  let showIconsDropdown = $state(false);
  let showColorsDropdown = $state(false);
  let showNestingDropdown = $state(false);
  let editName = $state('');
  let editParent = $state('');
  let editIcon = $state('Tag');
  let editColor = $state('blue');

  // Settings state
  let isSettingsOpen = $state(false);
  let showAccountDropdown = $state(false);

  let isDeletingConfirmOpen = $state(false);
  let deletingLabelName = $state('');
  let isCreatingLabelOpen = $state(false);
  let newLabelNameInput = $state('');
  let newLabelParentInput = $state('');

  let sidebarWidth = $state(256);
  let isResizing = $state(false);

  function handlePointerMove(e: PointerEvent) {
    if (isResizing) {
      sidebarWidth = Math.max(200, Math.min(e.clientX, 600));
    }
  }

  function handlePointerUp() {
    if (isResizing) {
      isResizing = false;
    }
  }

  // Compute nested/flattened labels
  let flattenedLabels = $derived(
    getFlattenedLabels(allLabels, collapsedLabels)
  );

  function toggleLabelCollapse(labelName: string, e: MouseEvent) {
    e.stopPropagation();
    collapsedLabels = {
      ...collapsedLabels,
      [labelName]: !collapsedLabels[labelName]
    };
  }

  function handleLabelContextMenu(labelName: string, e: MouseEvent) {
    e.preventDefault();
    const style = getLabelStyle(labelName, $labelCustomizations);
    const parts = labelName.split('/');
    editName = parts.pop() || '';
    editParent = parts.join('/');
    editIcon = style.iconName;
    editColor = style.colorName;
    
    showIconsDropdown = false;
    showColorsDropdown = false;
    showNestingDropdown = false;

    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      label: labelName
    };
  }

  function saveLabelCustomization(closeMenu = false) {
    if (!contextMenu) return;
    const currentLabel = contextMenu.label;
    let parentPath = editParent ? `${editParent}/` : '';
    const newFullName = parentPath + editName.trim();

    if (editName.trim() && newFullName !== currentLabel) {
      onRenameLabel(currentLabel, newFullName);
      labelCustomizations.update(c => {
        const updated = { ...c };
        delete updated[currentLabel];
        updated[newFullName] = { iconName: editIcon, colorName: editColor };
        return updated;
      });
      contextMenu.label = newFullName; // Update reference for subsequent saves
    } else {
      labelCustomizations.update(c => ({
        ...c,
        [currentLabel]: { iconName: editIcon, colorName: editColor }
      }));
    }
    if (closeMenu) {
      contextMenu = null;
    }
  }

  function handleCreateLabel() {
    if (newLabelNameInput.trim()) {
      const fullName = newLabelParentInput 
        ? `${newLabelParentInput}/${newLabelNameInput.trim()}`
        : newLabelNameInput.trim();
      onCreateNewLabel(fullName);
      newLabelNameInput = '';
      newLabelParentInput = '';
      isCreatingLabelOpen = false;
    }
  }

  onMount(() => {
    const handleGlobalClick = () => {
      if (contextMenu) contextMenu = null;
      showAccountDropdown = false;
    };
    window.addEventListener('click', handleGlobalClick);
    return () => window.removeEventListener('click', handleGlobalClick);
  });
</script>

<svelte:window onpointermove={handlePointerMove} onpointerup={handlePointerUp} />

<aside class="h-screen bg-[var(--color-canvas-card)] border-r border-[var(--color-border-hairline)] flex flex-col font-sans select-none shrink-0 relative transition-[width] duration-75" style="width: {sidebarWidth}px">

  <!-- Drag Handle -->
  <div 
    class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500/50 transition-colors z-50"
    onpointerdown={(e) => { isResizing = true; e.preventDefault(); }}
  ></div>

  <!-- Profile Header / Account Switcher at the Top -->
  <div id="profile-section" class="p-4 flex items-center justify-between mb-1 border-b border-[var(--color-border-hairline)]/30 shrink-0" data-tauri-drag-region>
    <div class="flex items-center gap-3 flex-1">
      <div 
        class="w-8 h-8 rounded-lg flex items-center justify-center font-mono text-sm font-semibold text-white select-none shrink-0" 
        style="background-color: {activeAccount?.color || 'rgba(255,255,255,0.1)'}"
      >
        {(activeAccount?.name || 'K')[0].toUpperCase()}
      </div>
      <div class="flex flex-col flex-1 relative">
        <Dropdown isOpen={showAccountDropdown} onClose={() => showAccountDropdown = false}>
          {#snippet trigger()}
            <button
              onclick={(e) => { e.stopPropagation(); showAccountDropdown = !showAccountDropdown; }}
              class="flex items-center gap-1.5 text-xs font-semibold text-white cursor-pointer hover:text-neutral-200 transition-colors w-full text-left justify-between pr-2"
            >
              <span class="truncate">{activeAccount?.name || ''}</span>
              <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)] shrink-0 transition-transform {showAccountDropdown ? 'rotate-180' : ''}" />
            </button>
          {/snippet}
          {#snippet content()}
            <div class="w-48 py-1 font-sans text-xs">
              {#each accounts as acc}
                <button
                  onclick={() => {
                    activeAccountId = acc.id;
                    showAccountDropdown = false;
                  }}
                  class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] transition-colors text-white cursor-pointer"
                >
                  <div class="w-2 h-2 rounded-full" style="background-color: {acc.color}"></div>
                  <div class="flex flex-col min-w-0">
                    <span class="font-semibold truncate">{acc.name}</span>
                    <span class="text-[9px] text-[var(--color-text-secondary)] truncate">{acc.email}</span>
                  </div>
                  {#if acc.id === activeAccountId}
                    <Check class="w-3 h-3 text-blue-400 ml-auto" />
                  {/if}
                </button>
              {/each}
            </div>
          {/snippet}
        </Dropdown>
        <span class="text-[10px] text-[var(--color-text-secondary)] truncate">{activeAccount?.email || ''}</span>
      </div>
    </div>
    
    <!-- Compose Button: Just an Icon Button -->
    <button 
      onclick={onComposeClick}
      title="Compose Message (C)"
      class="p-2 rounded-lg hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white cursor-pointer transition-colors active:scale-95 shrink-0"
    >
      <PenSquare class="w-4 h-4" />
    </button>
  </div>

  <!-- Search -->
  <div class="px-3 py-3 shrink-0">
    <div class="relative">
      <Search class="w-3.5 h-3.5 text-[var(--color-text-secondary)] absolute left-2.5 top-2.5" />
      <input
        type="text"
        placeholder="Search mail..."
        bind:value={searchQuery}
        class="w-full bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] text-xs rounded-lg pl-8 pr-4 py-2 outline-none border border-[var(--color-border-hairline)] focus:border-white/20 transition-all placeholder:text-[var(--color-text-secondary)]/40"
      />
    </div>
  </div>

  <!-- Nav Scroll Area -->
  <div class="flex-1 overflow-y-auto px-2 space-y-5 pb-2">

    <!-- Views Section Header -->
    <div>
      <div class="px-2.5 py-1 text-[10px] font-mono tracking-widest text-[var(--color-text-secondary)]/60 uppercase">
        Views
      </div>
      <div class="space-y-0.5 mt-1">
        {#each folders as folder}
          <button
            onclick={() => onSelectView(folder.id)}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 ease-in-out hover:translate-x-1
              {currentView === folder.id
                ? 'bg-[var(--color-canvas-hover)] text-white'
                : 'text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)]/60'}"
          >
            <div class="flex items-center gap-2">
              <folder.icon class="w-4 h-4 {folder.color}" strokeWidth={1.5} />
              <span>{folder.label}</span>
            </div>
            {#if (viewCounts[folder.id] ?? (folder.id === 'inbox' ? inboxCount : folder.id === 'unread' ? unreadCount : 0)) > 0}
              <span in:scale={{ duration: 200, start: 0.8 }} class="text-[9px] px-1.5 py-0.5 rounded-full font-bold bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] border border-[var(--color-border-hairline)]/40 shadow-sm">
                {viewCounts[folder.id] ?? (folder.id === 'inbox' ? inboxCount : unreadCount)}
              </span>
            {/if}
          </button>
        {/each}

        <!-- Categories Trigger -->
        <div>
          <button
            onclick={() => categoriesExpanded = !categoriesExpanded}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)]/60 transition-all duration-200 ease-in-out"
          >
            <div class="flex items-center gap-2">
              <Folder class="w-4 h-4 text-teal-400" strokeWidth={1.5} />
              <span>Categories</span>
            </div>
            {#if categoriesExpanded}
              <ChevronDown class="w-3 h-3 text-[var(--color-text-secondary)]" />
            {:else}
              <ChevronRight class="w-3 h-3 text-[var(--color-text-secondary)]" />
            {/if}
          </button>

          {#if categoriesExpanded}
            <div class="pl-6 space-y-0.5 mt-0.5" transition:slide={{duration: 200}}>
              {#each ['Primary', 'Updates', 'Social', 'Promotions', 'Forums'] as cat}
                <button
                  onclick={() => onSelectView(`category-${cat}`)}
                  class="w-full text-left px-2 py-1 text-[11px] rounded transition-colors text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]/40"
                >
                  {cat}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Labels Section Header -->
    <div>
      <div class="px-2.5 py-1 text-[10px] font-mono tracking-widest text-[var(--color-text-secondary)]/60 uppercase">
        Labels
      </div>

      <div class="space-y-0.5 mt-1">
        {#each flattenedLabels as item}
          {@const style = getLabelStyle(item.name, $labelCustomizations)}
          {@const IconComponent = iconMapping[style.iconName] || Tag}
          
          <button
            onclick={() => onSelectView(`label-${item.name}`)}
            oncontextmenu={(e) => handleLabelContextMenu(item.name, e)}
            draggable="true"
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 ease-in-out group hover:translate-x-1
              {currentView === `label-${item.name}`
                ? 'bg-[var(--color-canvas-hover)] text-white'
                : 'text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)]/60'}"
            style="padding-left: {item.depth * 12 + 10}px"
          >
            <div class="flex items-center gap-2 min-w-0">
              {#if item.hasChildren}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span 
                  onclick={(e) => toggleLabelCollapse(item.name, e)}
                  class="p-0.5 rounded hover:bg-white/10 shrink-0 cursor-pointer"
                  role="button"
                  tabindex="-1"
                >
                  {#if item.isExpanded}
                    <ChevronDown class="w-3 h-3 text-[var(--color-text-secondary)]" />
                  {:else}
                    <ChevronRight class="w-3 h-3 text-[var(--color-text-secondary)]" />
                  {/if}
                </span>
              {:else}
                <span class="w-4 shrink-0"></span>
              {/if}
              <IconComponent class="w-3.5 h-3.5 shrink-0 {style.textColor}" strokeWidth={1.5} />
              <span class="truncate text-[var(--color-text-primary)]">{item.displayName}</span>
            </div>
            {#if (viewCounts[`label-${item.name}`] ?? 0) > 0}
              <span class="bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] border border-[var(--color-border-hairline)]/40 text-[9px] px-1.5 py-0.5 rounded-full font-bold ml-auto">{viewCounts[`label-${item.name}`]}</span>
            {/if}
          </button>
        {/each}

        <!-- Add label trigger button -->
        <button
          onclick={() => isCreatingLabelOpen = true}
          class="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-canvas-hover)]/40 transition-colors cursor-pointer"
        >
          <span class="text-base leading-none -mt-0.5">+</span>
          <span>Add Label</span>
        </button>
      </div>
    </div>

  </div>

  <!-- Settings Footer -->
  <div class="p-3 bg-[var(--color-canvas-base)] flex flex-col gap-1 text-[var(--color-text-secondary)] border-t border-[var(--color-border-hairline)] shrink-0">
    <button 
      onclick={() => isSettingsOpen = true}
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-[var(--color-canvas-hover)]/60 transition-colors text-left cursor-pointer font-mono"
    >
      <Settings class="w-4 h-4" />
      <span>Settings</span>
    </button>
  </div>

  <!-- SETTINGS MODAL OVERLAY -->
  {#if isSettingsOpen}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs font-sans">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="fixed inset-0 cursor-pointer" onclick={() => isSettingsOpen = false} />
      
      <div class="relative w-full max-w-md bg-[#131313] border border-neutral-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden z-50 text-xs text-[var(--color-text-primary)]">
        <!-- Header -->
        <div class="px-5 py-4 border-b border-neutral-800/60 flex items-center justify-between bg-[#181818]">
          <div class="flex items-center gap-2">
            <Settings class="w-4 h-4 text-blue-400" />
            <h3 class="font-bold text-white uppercase tracking-wider">Mail Settings</h3>
          </div>
          <button onclick={() => isSettingsOpen = false} class="p-1 rounded hover:bg-white/10 text-neutral-400 hover:text-white transition-colors cursor-pointer">
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Scrollable Options -->
        <div class="p-6 space-y-4">
          <label class="flex items-center justify-between p-3 bg-neutral-900/35 border border-white/5 rounded-xl cursor-pointer">
            <div class="space-y-0.5">
              <span class="font-semibold text-white">Dense Layout Mode</span>
              <p class="text-[10px] text-[var(--color-text-secondary)]">Narrower heights for list elements.</p>
            </div>
            <input type="checkbox" bind:checked={$mailDenseMode} class="accent-blue-500 rounded cursor-pointer" />
          </label>

          <div class="space-y-1">
            <span class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Default Landing View</span>
            <select bind:value={$mailDefaultLandingView} class="w-full bg-[var(--color-canvas-base)] text-white rounded-lg p-2.5 outline-none border border-white/10 focus:border-white/20 transition-all cursor-pointer">
              <option value="inbox">Inbox Folder</option>
              <option value="unread">Unread Feed</option>
              <option value="starred">Starred List</option>
              <option value="all-mail">All Mail View</option>
            </select>
          </div>

          <div class="space-y-1">
            <span class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Email Signature</span>
            <textarea bind:value={$mailSignature} rows="3" class="w-full bg-[var(--color-canvas-base)] text-white rounded-lg p-2.5 outline-none border border-white/10 focus:border-white/20 transition-all resize-none font-sans" placeholder="Compose signature..."></textarea>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-5 py-3 border-t border-neutral-800/60 bg-[#181818] flex justify-end">
          <button onclick={() => isSettingsOpen = false} class="px-4 py-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 text-white font-semibold text-xs cursor-pointer transition-colors">Done</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Add New Label Modal -->
  {#if isCreatingLabelOpen}
    <div 
      class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4 font-sans"
      role="presentation"
      onclick={() => isCreatingLabelOpen = false}
    >
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div 
        class="bg-[#1a1919] border border-white/10 w-full max-w-xs rounded-xl shadow-xl p-4"
        onclick={(e) => e.stopPropagation()}
        role="dialog"
        tabindex="-1"
      >
        <h4 class="text-xs font-bold text-white uppercase tracking-wider mb-2">Create New Label</h4>
        <input 
          type="text" 
          placeholder="Folder name (e.g. Design)..."
          bind:value={newLabelNameInput}
          onkeydown={(e) => e.key === 'Enter' && handleCreateLabel()}
          class="w-full bg-[var(--color-canvas-base)] text-white text-xs rounded-lg p-2.5 outline-none border border-white/10 focus:border-white/20 transition-all mb-2"
        />
        <select bind:value={newLabelParentInput} style="color-scheme: dark;" class="w-full bg-[var(--color-canvas-base)] text-[var(--color-text-secondary)] text-xs rounded-lg p-2.5 outline-none border border-white/10 focus:border-white/20 transition-all mb-3 cursor-pointer leading-normal">
          <option value="">No parent (Top level)</option>
          {#each flattenedLabels as l}
            <option value={l.name} class="py-1">{l.name}</option>
          {/each}
        </select>
        <div class="flex justify-end gap-1.5">
          <button onclick={() => isCreatingLabelOpen = false} class="px-3 py-1.5 text-[11px] text-[var(--color-text-secondary)] hover:text-white">Cancel</button>
          <button onclick={handleCreateLabel} class="px-3.5 py-1.5 bg-white text-black font-semibold rounded-lg text-[11px] hover:bg-neutral-200 transition-colors">Create</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Label customization context menu -->
  {#if contextMenu}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="fixed bg-[#1a1919] border border-white/10 rounded-xl shadow-xl w-56 z-50 py-1.5 font-sans text-xs text-[var(--color-text-primary)]"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
      onclick={(e) => e.stopPropagation()}
      role="menu"
      tabindex="-1"
    >
      <div class="px-3 py-1.5 border-b border-white/5 flex items-center gap-1.5 shrink-0">
        <button
          onclick={() => {
            showIconsDropdown = !showIconsDropdown;
            showColorsDropdown = false;
            showNestingDropdown = false;
          }}
          class="p-1 rounded bg-[#1c1b1b] border border-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer shrink-0"
        >
          <svelte:component this={iconMapping[editIcon] || Tag} class="w-3.5 h-3.5 {colorConfigs[editColor]?.text || 'text-white'}" />
        </button>
        <input
          type="text"
          bind:value={editName}
          onkeydown={(e) => e.key === 'Enter' && saveLabelCustomization(true)}
          onblur={() => saveLabelCustomization(false)}
          class="w-full bg-transparent border-none text-xs text-white outline-none focus:ring-0 px-0.5 py-0.5"
          placeholder="Label name"
          autoFocus
        />
      </div>

      <!-- Icon Select Dropdown -->
      {#if showIconsDropdown}
        <div class="p-2 border-b border-white/5 grid grid-cols-6 gap-1 max-h-32 overflow-y-auto">
          {#each Object.keys(iconMapping) as iconKey}
            {@const IconComponent = iconMapping[iconKey]}
            <button
              onclick={() => { editIcon = iconKey; showIconsDropdown = false; saveLabelCustomization(false); }}
              class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer flex items-center justify-center border-none bg-transparent"
            >
              <IconComponent class="w-3.5 h-3.5" strokeWidth={1.5} />
            </button>
          {/each}
        </div>
      {/if}

      <!-- Color Select Trigger -->
      <button
        onclick={() => {
          showColorsDropdown = !showColorsDropdown;
          showIconsDropdown = false;
          showNestingDropdown = false;
        }}
        class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors border-none bg-transparent"
      >
        <span class="w-2.5 h-2.5 rounded-full shrink-0 {colorConfigs[editColor]?.dot || 'bg-white'}"></span>
        <span>Change Label Color</span>
      </button>

      {#if showColorsDropdown}
        <div class="px-3 py-1.5 border-b border-white/5 flex flex-wrap gap-1.5 justify-center">
          {#each Object.keys(colorConfigs) as colorKey}
            <button
              onclick={() => { editColor = colorKey; showColorsDropdown = false; saveLabelCustomization(false); }}
              class="w-5 h-5 rounded-full {colorConfigs[colorKey].dot} cursor-pointer transition-transform hover:scale-110 border-none"
              style="box-shadow: {editColor === colorKey ? '0 0 6px rgba(255,255,255,0.6)' : 'none'}"
            />
          {/each}
        </div>
      {/if}

      <!-- Parent Select Trigger -->
      <button
        onclick={() => {
          showNestingDropdown = !showNestingDropdown;
          showColorsDropdown = false;
          showIconsDropdown = false;
        }}
        class="w-full px-3 py-2 text-left hover:bg-[var(--color-canvas-hover)] flex items-center gap-2 cursor-pointer transition-colors border-none bg-transparent"
      >
        <Folder class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" strokeWidth={1.5} />
        <span>Nest under...</span>
      </button>

      {#if showNestingDropdown}
        <div class="p-2 border-b border-white/5 flex flex-col gap-1 max-h-40 overflow-y-auto">
          <button onclick={() => { editParent = ''; showNestingDropdown = false; saveLabelCustomization(false); }} class="w-full block shrink-0 text-left text-xs px-2 py-1.5 hover:bg-white/10 rounded border-none bg-transparent {editParent === '' ? 'text-blue-400' : 'text-white'}">Top level</button>
          {#each flattenedLabels.filter(l => l.name !== contextMenu?.label && !l.name.startsWith((contextMenu?.label || '') + '/')) as l}
            <button onclick={() => { editParent = l.name; showNestingDropdown = false; saveLabelCustomization(false); }} class="w-full block shrink-0 text-left text-xs px-2 py-1.5 hover:bg-white/10 rounded truncate border-none bg-transparent {editParent === l.name ? 'text-blue-400' : 'text-white'}">
              {l.name}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Delete trigger -->
      <button
        onclick={() => {
          onDeleteLabel(contextMenu!.label);
          contextMenu = null;
        }}
        class="w-full px-3 py-2 text-left hover:bg-red-500/10 text-red-400 flex items-center gap-2 cursor-pointer transition-colors border-t border-white/5 mt-1 border-none bg-transparent"
      >
        <Trash2 class="w-3.5 h-3.5" />
        <span>Delete Folder</span>
      </button>
    </div>
  {/if}

</aside>
