<script lang="ts">
  import { 
    Plus, ChevronLeft, ChevronRight, Settings, Eye, EyeOff, Check, ChevronDown,
    CalendarDays, Calendar as CalendarIcon, Clock, AlignLeft
  } from 'lucide-svelte';

  export interface Calendar {
    id: string;
    name: string;
    color: string;
    isActive: boolean;
    isDefault?: boolean;
  }

  export interface Account {
    id: string;
    email: string;
    isExpanded: boolean;
    calendars: Calendar[];
  }

  let {
    selectedDate = new Date(),
    onDateSelect = (d: Date) => {},
    onNewEventClick = () => {},
    accounts = $bindable([] as Account[]),
    onToggleCalendar = (accId: string, calId: string) => {},
    events = [] as any[],
    isMobileOrTablet = false,
    viewMode = 'month',
    onViewModeChange = (mode: string) => {}
  } = $props<{
    selectedDate?: Date;
    onDateSelect?: (d: Date) => void;
    onNewEventClick?: () => void;
    accounts?: Account[];
    onToggleCalendar?: (accId: string, calId: string) => void;
    events?: any[];
    isMobileOrTablet?: boolean;
    viewMode?: string;
    onViewModeChange?: (mode: string) => void;
  }>();

  let currentMonth = $state(new Date());

  // Helper to build recursive days in the mini-month grid
  let daysInMonth = $derived(() => {
    const year = currentMonth.getFullYear();
    const month = currentMonth.getMonth();
    
    // First day of the month index (0 = Sun, 6 = Sat)
    const firstDay = new Date(year, month, 1).getDay();
    const daysCount = new Date(year, month + 1, 0).getDate();
    
    const days = [];
    // Pad previous month empty slots
    for (let i = 0; i < firstDay; i++) days.push(null);
    // Add current month days
    for (let d = 1; d <= daysCount; d++) days.push(new Date(year, month, d));
    return days;
  });

  function handlePrevMonth() {
    currentMonth = new Date(currentMonth.getFullYear(), currentMonth.getMonth() - 1, 1);
  }

  function handleNextMonth() {
    currentMonth = new Date(currentMonth.getFullYear(), currentMonth.getMonth() + 1, 1);
  }

  // Swipe logic for mini-month picker
  let touchStartX = $state(0);
  let touchEndX = $state(0);
  
  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
  }
  
  function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].clientX;
    const threshold = 30;
    if (touchEndX < touchStartX - threshold) handleNextMonth(); // swipe left
    if (touchEndX > touchStartX + threshold) handlePrevMonth(); // swipe right
  }

  function toggleAccount(accId: string) {
    accounts = accounts.map((a: Account) => a.id === accId ? { ...a, isExpanded: !a.isExpanded } : a);
  }

  const COLOR_HEX: Record<string, string> = {
    blue: '#2383e2',
    purple: '#8a4bf5',
    green: '#0fa35c',
    orange: '#df6a14',
    rose: '#e03e3e',
    amber: '#dfab00',
    teal: '#0fa3b1'
  };
</script>

<aside class="w-64 h-screen bg-[var(--color-canvas-card)] border-r border-[var(--color-border-hairline)] flex flex-col font-sans select-none p-4 space-y-5 shrink-0">
  
  <!-- Header with New Event Button -->
  <div class="flex items-center justify-between border-b border-[var(--color-border-hairline)]/40 pb-3 shrink-0" data-tauri-drag-region>
    <div class="flex items-center gap-2 font-bold text-sm text-white">
      <span class="w-7 h-7 rounded-lg bg-blue-500/20 text-blue-400 flex items-center justify-center font-mono">C</span>
      <span>Calendar</span>
    </div>

    <button
      onclick={onNewEventClick}
      class="px-3 py-1.5 rounded-lg bg-white text-black text-xs font-semibold hover:bg-neutral-200 transition-all duration-300 flex items-center gap-1 cursor-pointer active:scale-95 shadow-[0_4px_14px_0_rgba(255,255,255,0.2)] hover:shadow-[0_6px_20px_rgba(255,255,255,0.3)]"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>New Event</span>
    </button>
  </div>

  <!-- Mini Month Picker (Styled like Prototype) -->
  <div class="bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-xl p-3 shrink-0"
       ontouchstart={handleTouchStart}
       ontouchend={handleTouchEnd}
       role="region" aria-label="Mini Calendar">
    <div class="flex items-center justify-between mb-3 text-xs font-semibold text-white">
      <span>{currentMonth.toLocaleString('default', { month: 'long', year: 'numeric' })}</span>
      <div class="flex items-center gap-1">
        <button
          onclick={handlePrevMonth}
          class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
        >
          <ChevronLeft class="w-3.5 h-3.5" />
        </button>
        <button
          onclick={handleNextMonth}
          class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
        >
          <ChevronRight class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <div class="grid grid-cols-7 gap-1 text-center text-[10px] font-mono text-[var(--color-text-secondary)]/50 font-medium mb-1.5">
      <span>Su</span><span>Mo</span><span>Tu</span><span>We</span><span>Th</span><span>Fr</span><span>Sa</span>
    </div>

    <div class="grid grid-cols-7 gap-1 text-center text-xs font-mono">
      {#each daysInMonth() as day}
        {#if day}
          {@const isSelected = day.toDateString() === selectedDate.toDateString()}
          {@const isToday = day.toDateString() === new Date().toDateString()}
          {@const hasEvent = events.some((e: any) => e.date === day.toISOString().split('T')[0])}
          <button
            onclick={() => { onDateSelect(day!); currentMonth = day!; }}
            class="h-6 w-full rounded flex flex-col items-center justify-center transition-all cursor-pointer relative text-[11px]
              {isToday 
                ? 'bg-[var(--color-today-red)] text-white font-semibold shadow-sm' 
                : isSelected 
                ? 'border border-white/20 bg-[var(--color-canvas-hover)] text-white font-semibold' 
                : 'text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)]'}"
          >
            <span>{day.getDate()}</span>
            {#if hasEvent && !isSelected && !isToday}
              <div class="w-1 h-1 bg-[var(--color-text-secondary)] rounded-full absolute bottom-0.5"></div>
            {:else if hasEvent}
              <div class="w-1 h-1 bg-white rounded-full absolute bottom-0.5"></div>
            {/if}
          </button>
        {:else}
          <div></div>
        {/if}
      {/each}
    </div>
  </div>

  <!-- Accounts & Calendars Collapsible List (Matching Prototype) -->
  <div class="flex-1 overflow-y-auto space-y-4 pr-1">
    
    {#if isMobileOrTablet}
      <div id="sidebar-view-modes" class="space-y-3 pt-1">
        <h3 class="px-1 text-[10px] font-mono tracking-wider text-[var(--color-text-secondary)]/60 uppercase font-semibold">
          View Modes
        </h3>
        <div class="space-y-1">
          {#each [
            { label: 'Month', mode: 'month', icon: CalendarDays },
            { label: 'Week', mode: 'week', icon: CalendarIcon },
            { label: 'Day', mode: 'day', icon: Clock },
            { label: 'Agenda', mode: 'agenda', icon: AlignLeft },
          ] as item}
            <button
              onclick={() => onViewModeChange(item.mode)}
              class="w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-xs font-mono transition-colors cursor-pointer {viewMode === item.mode ? 'bg-white text-black font-semibold' : 'text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]'}"
            >
              <div class="flex items-center gap-2.5">
                <svelte:component this={item.icon} class="w-4 h-4 shrink-0" />
                <span>{item.label}</span>
              </div>
              {#if viewMode === item.mode}
                <Check class="w-3.5 h-3.5 stroke-[2.5]" />
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <div class="space-y-3 pt-1">
      <div class="text-[10px] font-mono uppercase tracking-wider text-[var(--color-text-secondary)]/60 font-semibold px-1">
        Calendars & Accounts
      </div>
      {#each accounts as acc}
        <div class="space-y-1.5">
          <!-- Account Toggle Row -->
          <button
            onclick={() => toggleAccount(acc.id)}
            class="w-full flex items-center justify-between px-1 py-1 rounded text-left text-xs font-medium text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]/40 transition-colors cursor-pointer group"
          >
            <span class="truncate pr-2 font-mono text-[11px]" title={acc.email}>
              {acc.email}
            </span>
            <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)]/60 transition-transform {acc.isExpanded ? '' : '-rotate-90'}" />
          </button>

          <!-- Calendars List nested -->
          {#if acc.isExpanded}
            <div class="space-y-0.5 pl-2.5 animate-slideDown">
              {#each acc.calendars as cal}
                {@const hexColor = COLOR_HEX[cal.color] || '#3B82F6'}
                <div class="w-full flex items-center justify-between py-1 px-1.5 rounded text-xs transition-colors hover:bg-[var(--color-canvas-hover)]/30">
                  <button
                    onclick={() => onToggleCalendar(acc.id, cal.id)}
                    class="flex items-center gap-2.5 text-left text-[var(--color-text-primary)] hover:text-white cursor-pointer flex-1 truncate"
                  >
                    <!-- Colored Checkbox Box -->
                    <div 
                      class="w-3.5 h-3.5 rounded border transition-all flex items-center justify-center shrink-0"
                      style="border-color: {cal.isActive ? 'transparent' : 'var(--color-border-hairline)'}; background-color: {cal.isActive ? hexColor : 'transparent'}"
                    >
                      {#if cal.isActive}
                        <Check class="w-2.5 h-2.5 text-black stroke-[3]" />
                      {/if}
                    </div>
                    <span class="truncate text-[var(--color-text-primary)]/80 font-medium text-xs">
                      {cal.name}
                    </span>
                    {#if cal.isDefault}
                      <span class="text-[8px] font-mono text-[var(--color-text-secondary)]/40 border border-[var(--color-border-hairline)] px-1 rounded shrink-0">
                        Default
                      </span>
                    {/if}
                  </button>

                  <!-- Visibility Toggles -->
                  <button
                    onclick={() => onToggleCalendar(acc.id, cal.id)}
                    class="p-0.5 rounded text-[var(--color-text-secondary)]/40 hover:text-[var(--color-text-secondary)] transition-colors cursor-pointer ml-1"
                    title={cal.isActive ? 'Hide' : 'Show'}
                  >
                    {#if cal.isActive}
                      <Eye class="w-3.5 h-3.5 text-[var(--color-text-secondary)]/60" />
                    {:else}
                      <EyeOff class="w-3.5 h-3.5 text-[var(--color-text-secondary)]/25" />
                    {/if}
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Settings Footer -->
  <div class="pt-3 border-t border-[var(--color-border-hairline)] flex items-center justify-between text-xs text-[var(--color-text-secondary)] shrink-0 bg-[var(--color-canvas-card)]">
    <button class="flex items-center gap-2 hover:text-white transition-colors cursor-pointer w-full text-left font-mono">
      <Settings class="w-4 h-4" />
      <span>Settings</span>
    </button>
  </div>
</aside>
