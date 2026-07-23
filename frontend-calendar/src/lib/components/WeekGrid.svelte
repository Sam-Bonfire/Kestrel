<script lang="ts">
  import { Clock, MapPin, AlignLeft, CalendarDays, Calendar as CalendarIcon, CheckSquare } from 'lucide-svelte';

  export interface CalendarEvent {
    id: string;
    title: string;
    date: string; // YYYY-MM-DD
    startTime: string; // HH:MM
    endTime: string; // HH:MM
    description?: string;
    color: string; // 'blue' | 'purple' | 'green' | 'orange' | 'rose' | 'amber' | 'teal'
    location?: string;
    category?: string;
    calendarId: string;
    status?: string;
    priority?: string;
    isAllDay?: boolean;
    organizer?: string;
    attendees?: { name: string; email: string; rsvp: 'yes' | 'no' | 'maybe' }[];
    rsvpStatus?: 'yes' | 'no' | 'maybe';
  }

  let {
    events = [] as CalendarEvent[],
    selectedDate = new Date(),
    viewMode = $bindable('month'),
    showWeekends = true,
    startHour = 8,
    selectedEventId = null as string | null,
    onEventClick = (ev: CalendarEvent) => {},
    onEmptySlotClick = () => {},
    onChangeViewMode = () => {}
  } = $props<{
    events?: CalendarEvent[];
    selectedDate?: Date;
    viewMode?: string;
    showWeekends?: boolean;
    startHour?: number;
    selectedEventId?: string | null;
    onEventClick?: (ev: CalendarEvent, e?: MouseEvent) => void;
    onEmptySlotClick?: (dateStr: string, timeStr: string, e?: MouseEvent) => void;
    onChangeViewMode?: (v: string) => void;
  }>();

  // Full 24-hour timeline
  const hours = Array.from({ length: 24 }, (_, i) => i);
  
  // Timezone string calculation (e.g. GMT-4)
  let tzOffsetStr = $derived(() => {
    const offset = -(new Date().getTimezoneOffset()) / 60;
    return `GMT${offset >= 0 ? '+' : ''}${offset}`;
  });

  // Color options configurations matching design tokens and presets
  const COLOR_CLASSES: Record<string, { bg: string; border: string; hex: string }> = {
    blue: { bg: 'bg-blue-500/20 text-blue-200 border-l-[3px] border-l-blue-500', border: 'border-blue-400', hex: '#3b82f6' },
    purple: { bg: 'bg-purple-500/20 text-purple-200 border-l-[3px] border-l-purple-500', border: 'border-purple-400', hex: '#a855f7' },
    green: { bg: 'bg-emerald-500/20 text-emerald-200 border-l-[3px] border-l-emerald-500', border: 'border-emerald-400', hex: '#10b981' },
    orange: { bg: 'bg-orange-500/20 text-orange-200 border-l-[3px] border-l-orange-500', border: 'border-orange-400', hex: '#f97316' },
    rose: { bg: 'bg-rose-500/20 text-rose-200 border-l-[3px] border-l-rose-500', border: 'border-rose-400', hex: '#f43f5e' },
    amber: { bg: 'bg-amber-500/20 text-amber-200 border-l-[3px] border-l-amber-500', border: 'border-amber-400', hex: '#f59e0b' },
    teal: { bg: 'bg-teal-500/20 text-teal-200 border-l-[3px] border-l-teal-500', border: 'border-teal-400', hex: '#14b8a6' },
  };

  // Helper to format ISO date string YYYY-MM-DD
  function toISODateString(d: Date): string {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  // Get start of the week for date
  function getStartOfWeek(d: Date): Date {
    const day = d.getDay();
    const diff = d.getDate() - day;
    return new Date(d.setDate(diff));
  }

  // Generate range of dates based on view mode
  let visibleDates = $derived(() => {
    const dates: Date[] = [];
    const nDayMatch = viewMode.match(/^(\d+)-day$/);
    
    if (viewMode === 'day') {
      dates.push(new Date(selectedDate));
    } else if (nDayMatch) {
      const days = parseInt(nDayMatch[1], 10);
      for (let i = 0; i < days; i++) {
        dates.push(new Date(selectedDate.getFullYear(), selectedDate.getMonth(), selectedDate.getDate() + i));
      }
    } else if (viewMode === 'week') {
      const start = getStartOfWeek(new Date(selectedDate));
      for (let i = 0; i < 7; i++) {
        dates.push(new Date(start.getFullYear(), start.getMonth(), start.getDate() + i));
      }
    } else if (viewMode === 'weekdays') {
      const start = getStartOfWeek(new Date(selectedDate));
      for (let i = 1; i <= 5; i++) {
        dates.push(new Date(start.getFullYear(), start.getMonth(), start.getDate() + i));
      }
    } else if (viewMode === 'month') {
      // 35 days month calendar
      const year = selectedDate.getFullYear();
      const month = selectedDate.getMonth();
      const firstDayOffset = new Date(year, month, 1).getDay();
      const startDate = new Date(year, month, 1 - firstDayOffset);
      for (let i = 0; i < 35; i++) {
        dates.push(new Date(startDate.getFullYear(), startDate.getMonth(), startDate.getDate() + i));
      }
    }
    
    // Filter out weekends if required (0 = Sunday, 6 = Saturday)
    return showWeekends ? dates : dates.filter(d => d.getDay() !== 0 && d.getDay() !== 6);
  });

  // Time layout calculations
  function getEventTopOffset(startTime: string): number {
    const [h, m] = startTime.split(':').map(Number);
    const minSinceMidnight = h * 60 + m;
    // 1 hour is 60px height, so 1 minute is 1px height
    return minSinceMidnight;
  }

  function getEventHeight(startTime: string, endTime: string): number {
    const [sh, sm] = startTime.split(':').map(Number);
    const [eh, em] = endTime.split(':').map(Number);
    const durationMins = (eh - sh) * 60 + (em - sm);
    return Math.max(durationMins, 30); // minimum 30px height
  }

  // Calculate overlapping events layout for a day
  function getTimedEventsLayout(eventsForDay: CalendarEvent[]) {
    const sorted = [...eventsForDay].sort((a, b) => {
      const startDiff = getEventTopOffset(a.startTime) - getEventTopOffset(b.startTime);
      if (startDiff !== 0) return startDiff;
      return getEventTopOffset(b.endTime) - getEventTopOffset(a.endTime);
    });

    const columns: CalendarEvent[][] = [];
    const layout = new Map<string, { col: number; maxCol: number }>();

    for (const ev of sorted) {
      let placed = false;
      for (let i = 0; i < columns.length; i++) {
        const colLastEv = columns[i][columns[i].length - 1];
        if (getEventTopOffset(colLastEv.endTime) <= getEventTopOffset(ev.startTime)) {
          columns[i].push(ev);
          layout.set(ev.id, { col: i, maxCol: 0 });
          placed = true;
          break;
        }
      }
      if (!placed) {
        columns.push([ev]);
        layout.set(ev.id, { col: columns.length - 1, maxCol: 0 });
      }
    }

    for (const ev of sorted) {
      const data = layout.get(ev.id);
      if (data) data.maxCol = columns.length;
    }

    return layout;
  }

  // Filter events by YYYY-MM-DD string
  function getEventsForDate(dateStr: string): CalendarEvent[] {
    return events.filter((e: CalendarEvent) => e.date === dateStr);
  }

  import { onMount } from 'svelte';
  let scrollContainer: HTMLDivElement | null = $state(null);

  $effect(() => {
    // Auto-scroll timeline to startHour
    if (scrollContainer && viewMode !== 'month' && viewMode !== 'agenda') {
      scrollContainer.scrollTop = startHour * 60;
    }
  });
</script>

<div class="flex-1 h-screen bg-[var(--color-canvas-base)] flex flex-col font-sans overflow-hidden">
  
  {#if viewMode === 'agenda'}
    <!-- Agenda List Layout -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6">
      <div class="flex items-center gap-2 border-b border-[var(--color-border-hairline)] pb-3">
        <AlignLeft class="w-4 h-4 text-blue-400" />
        <h2 class="text-sm font-semibold text-white">Chronological Agenda View</h2>
      </div>

      {#if events.length === 0}
        <div class="flex flex-col items-center justify-center py-20 text-[var(--color-text-secondary)] gap-2">
          <Clock class="w-8 h-8 opacity-20" />
          <span class="text-xs opacity-50">No upcoming events</span>
        </div>
      {:else}
        <div class="space-y-4 max-w-2xl">
          {#each [...events].sort((a,b) => a.date.localeCompare(b.date) || a.startTime.localeCompare(b.startTime)) as ev}
            {@const style = COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue}
            <button
              onclick={(e) => onEventClick(ev, e)}
              class="w-full flex gap-4 p-4 border border-[var(--color-border-hairline)] rounded-xl hover:border-white/20 transition-all text-left bg-[#131313]/20 cursor-pointer"
            >
              <div class="w-2.5 shrink-0 rounded-full" style="background-color: {(COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue).hex}"></div>
              <div class="space-y-1 min-w-0 flex-1">
                <div class="flex items-baseline justify-between">
                  <span class="text-sm font-bold text-white truncate">{ev.title}</span>
                  <span class="text-[10px] font-mono text-[var(--color-text-secondary)] shrink-0">{ev.date}</span>
                </div>
                <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-[var(--color-text-secondary)]">
                  <span class="flex items-center gap-1">
                    <Clock class="w-3.5 h-3.5" />
                    <span>{ev.startTime} - {ev.endTime}</span>
                  </span>
                  {#if ev.location}
                    <span class="flex items-center gap-1 truncate max-w-[200px]">
                      <MapPin class="w-3.5 h-3.5" />
                      <span class="truncate">{ev.location}</span>
                    </span>
                  {/if}
                </div>
                {#if ev.description}
                  <p class="text-[11px] text-[var(--color-text-secondary)]/80 line-clamp-2 pt-1">{ev.description}</p>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

  {:else if viewMode === 'month'}
    <!-- Month Grid View (35 cells) -->
    <div class="grid grid-cols-7 border-b border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)] text-xs font-semibold text-[var(--color-text-secondary)] shrink-0 select-none">
      {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
        <div class="p-3 text-center border-r border-[var(--color-border-hairline)] last:border-r-0">
          {day}
        </div>
      {/each}
    </div>

    <div class="flex-1 grid grid-cols-7 grid-rows-5 divide-x divide-y divide-[var(--color-border-hairline)]/30 overflow-hidden bg-[var(--color-canvas-base)]">
      {#each visibleDates() as date}
        {@const dateStr = toISODateString(date)}
        {@const isToday = date.toDateString() === new Date().toDateString()}
        {@const isCurrentMonth = date.getMonth() === selectedDate.getMonth()}
        <div class="p-2 border-r border-b border-[var(--color-border-hairline)]/20 flex flex-col hover:bg-[var(--color-canvas-hover)]/10 transition-colors min-h-0 overflow-hidden">
          <div class="flex justify-between items-center mb-1">
            <span class="text-[10px] font-mono font-semibold 
              {isToday 
                ? 'bg-[var(--color-today-red)] text-white px-1.5 py-0.5 rounded' 
                : isCurrentMonth ? 'text-[var(--color-text-primary)]' : 'text-[var(--color-text-secondary)]/30'}">
              {date.getDate()}
            </span>
          </div>

          <div class="flex-1 space-y-1 overflow-y-auto pr-0.5">
            {#each getEventsForDate(dateStr) as ev}
              <button
                onclick={(e) => onEventClick(ev, e)}
                class="w-full px-2 py-0.5 rounded text-[10px] text-left font-medium truncate shadow-sm cursor-pointer transition-transform hover:scale-[1.01] {(COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue).bg} {ev.id === selectedEventId ? 'ring-2 ring-white ring-offset-2 ring-offset-[#131313] z-10' : 'border-transparent'}"
              >
                {ev.title}
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </div>

  {:else}
    <!-- Timeline Views (Week, Weekdays, Day) -->
    <div class="grid border-b border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)] text-xs font-semibold text-[var(--color-text-secondary)] shrink-0 select-none"
         style="grid-template-columns: 64px repeat({visibleDates().length}, minmax(0, 1fr));">
      <div class="p-3 border-r border-[var(--color-border-hairline)] font-mono text-[10px] flex flex-col items-center justify-center">
        <span class="opacity-70">{tzOffsetStr()}</span>
      </div>
      {#each visibleDates() as date}
        {@const isToday = date.toDateString() === new Date().toDateString()}
        <div class="p-3 text-center border-r border-[var(--color-border-hairline)] last:border-r-0 flex flex-col items-center">
          <span class="text-[10px] text-[var(--color-text-secondary)]/70 font-mono uppercase">
            {date.toLocaleDateString(undefined, { weekday: 'short' })}
          </span>
          <span class="text-xs font-bold mt-0.5 
            {isToday 
              ? 'bg-[var(--color-today-red)] text-white px-1.5 py-0.5 rounded' 
              : 'text-white'}">
            {date.getDate()}
          </span>
        </div>
      {/each}
    </div>

    <!-- Timeline scroll container -->
    <div bind:this={scrollContainer} class="flex-1 overflow-y-auto relative bg-[var(--color-canvas-base)]">
      
      <!-- Hour horizontal grid lines overlay -->
      <div class="absolute inset-0 pointer-events-none divide-y divide-[var(--color-border-hairline)]/20">
        {#each hours as _}
          <div class="h-[60px]"></div>
        {/each}
      </div>

      <!-- Time blocks columns -->
      <div class="relative min-h-[1440px] grid" 
           style="grid-template-columns: 64px repeat({visibleDates().length}, minmax(0, 1fr));">
        
        <!-- Y-Axis Hours list labels -->
        <div class="border-r border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)]/10">
          {#each hours as hour}
            <div class="h-[60px] p-2 text-[10px] font-mono text-[var(--color-text-secondary)]/60 text-right">
              {hour === 0 ? '12 AM' : hour === 12 ? '12 PM' : hour > 12 ? `${hour - 12} PM` : `${hour} AM`}
            </div>
          {/each}
        </div>

        <!-- Columns for each day -->
        {#each visibleDates() as date}
          {@const dateStr = toISODateString(date)}
          {@const dayEvents = getEventsForDate(dateStr)}
          {@const layoutInfo = getTimedEventsLayout(dayEvents)}
          <div class="border-r border-[var(--color-border-hairline)]/30 last:border-r-0 relative hover:bg-[var(--color-canvas-hover)]/5 transition-colors"
               onclick={(e) => {
                 // Fractional time selection (15 minute intervals)
                 const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                 const y = e.clientY - rect.top;
                 
                 // Calculate exact hour and nearest 15 minute fraction
                 const exactHour = Math.floor(y / 60);
                 const exactMinutes = y % 60;
                 const roundedMinutes = Math.floor(exactMinutes / 15) * 15;
                 
                 const timeStr = `${exactHour.toString().padStart(2, '0')}:${roundedMinutes.toString().padStart(2, '0')}`;
                 onEmptySlotClick?.(dateStr, timeStr, e);
               }}
               role="button"
               tabindex="0"
               onkeydown={(e) => { if (e.key === 'Enter') onEmptySlotClick?.(dateStr, '09:00'); }}
          >
            
            <!-- Render events for this column day -->
            {#each dayEvents as ev}
              {@const top = getEventTopOffset(ev.startTime)}
              {@const height = getEventHeight(ev.startTime, ev.endTime)}
              {@const colStyle = COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue}
              {@const layout = layoutInfo.get(ev.id)}
              {@const widthPct = layout ? 100 / layout.maxCol : 100}
              {@const leftPct = layout ? (100 / layout.maxCol) * layout.col : 0}
              
              <button
                onclick={(e) => { e.stopPropagation(); onEventClick(ev, e); }}
                class="absolute p-2 rounded-lg text-xs text-left font-medium border shadow-md transition-transform hover:scale-[1.02] cursor-pointer overflow-hidden
                  {colStyle.bg} {colStyle.border} {ev.id === selectedEventId ? 'ring-2 ring-white ring-offset-2 ring-offset-[#131313] z-50' : 'z-10'}"
                style="top: {top}px; height: {height}px; left: calc({leftPct}% + 4px); width: calc({widthPct}% - 8px);"
              >
                <div class="font-bold truncate text-white leading-tight">{ev.title}</div>
                <div class="text-[10px] opacity-75 font-mono mt-0.5">{ev.startTime} - {ev.endTime}</div>
                {#if ev.location && height > 50}
                  <div class="text-[9px] opacity-80 truncate flex items-center gap-1 mt-1">
                    <MapPin class="w-3 h-3 text-current shrink-0" />
                    <span class="truncate">{ev.location}</span>
                  </div>
                {/if}
              </button>
            {/each}
          </div>
        {/each}

      </div>
    </div>
  {/if}
</div>
