<script lang="ts">
  import { Clock, MapPin, AlignLeft, CalendarDays, Calendar as CalendarIcon, CheckSquare } from 'lucide-svelte';
  import { scale } from 'svelte/transition';
  import EventHoverPopover from './EventHoverPopover.svelte';

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
    secondaryTimezones = [] as string[],
    selectedEventId = null as string | null,
    onEventClick = (ev: CalendarEvent) => {},
    onEmptySlotClick = () => {},
    onChangeViewMode = () => {},
    onEventUpdate = (id: string, updates: Partial<CalendarEvent>) => {}
  } = $props<{
    events?: CalendarEvent[];
    selectedDate?: Date;
    viewMode?: string;
    showWeekends?: boolean;
    startHour?: number;
    secondaryTimezones?: string[];
    selectedEventId?: string | null;
    onEventClick?: (ev: CalendarEvent, e?: MouseEvent) => void;
    onEmptySlotClick?: (dateStr: string, timeStr: string, e?: MouseEvent) => void;
    onChangeViewMode?: (v: string) => void;
    onEventUpdate?: (id: string, updates: Partial<CalendarEvent>) => void;
  }>();

  // Full 24-hour timeline
  const hours = Array.from({ length: 24 }, (_, i) => i);
  
  // Timezone string calculation (e.g. GMT-4)
  let tzOffsetStr = $derived(() => {
    const offset = -(new Date().getTimezoneOffset()) / 60;
    return `GMT${offset >= 0 ? '+' : ''}${offset}`;
  });

  // Helper to format hours in a specific timezone
  function formatHourInTimezone(hour: number, tz: string): string {
    const d = new Date();
    d.setHours(hour, 0, 0, 0);
    try {
      const parts = new Intl.DateTimeFormat('en-US', {
        timeZone: tz,
        hour: 'numeric',
        hour12: true
      }).formatToParts(d);
      return parts.map(p => p.value).join('');
    } catch(e) {
      return `${hour}:00`;
    }
  }

  // Calculate the gutter width based on number of timezones
  let gutterWidth = $derived(64 + (secondaryTimezones.length * 56));

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
    // 1. Convert times to fractional hours for overlap math
    const evts = eventsForDay.map(e => {
      const [sh, sm] = (e.startTime || '09:00').split(':').map(Number);
      const start = sh + sm / 60;
      const [eh, em] = (e.endTime || '10:00').split(':').map(Number);
      const end = Math.max(start + 0.5, eh + em / 60);
      return { ...e, start, end };
    });

    // 2. Group overlapping events
    const groups: typeof evts[] = [];
    evts.sort((a, b) => a.start - b.start || b.end - a.end).forEach(evt => {
      let added = false;
      for (const g of groups) {
        const overlapsAny = g.some(other => (evt.start < other.end && evt.end > other.start));
        if (overlapsAny) {
          g.push(evt);
          added = true;
          break;
        }
      }
      if (!added) groups.push([evt]);
    });

    // 3. Assign columns and calculate percentages
    const layout = new Map<string, { left: number; width: number; col: number; maxCol: number }>();
    groups.forEach(g => {
      const columns: string[][] = [];
      g.forEach(evt => {
        let colIdx = 0;
        while (true) {
          if (!columns[colIdx]) columns[colIdx] = [];
          const hasOverlap = columns[colIdx].some(otherId => {
            const other = g.find(o => o.id === otherId)!;
            return (evt.start < other.end && evt.end > other.start);
          });
          if (!hasOverlap) {
            columns[colIdx].push(evt.id);
            break;
          }
          colIdx++;
        }
      });
      
      const totalCols = columns.length;
      g.forEach(evt => {
        const colIdx = columns.findIndex(col => col.includes(evt.id));
        layout.set(evt.id, {
          left: (colIdx / totalCols) * 100,
          width: 100 / totalCols,
          col: colIdx,
          maxCol: totalCols
        });
      });
    });

    return layout;
  }

  // Filter events by YYYY-MM-DD string
  function getEventsForDate(dateStr: string): CalendarEvent[] {
    return events.filter((e: CalendarEvent) => e.date === dateStr);
  }

  // ── Drag-to-create state & helpers ──────────────────────────────
  let dragCreate = $state<{
    dateStr: string;
    startY: number;
    currentY: number;
    active: boolean;
  } | null>(null);
  // Set when a drag-create committed, so the subsequent `click` event is ignored
  let suppressNextClick = $state(false);

  // Convert a Y coordinate (relative to the timeline body) into HH:MM,
  // snapped to 15-minute increments and clamped to 23:59.
  function yToTime(y: number): string {
    const clamped = Math.max(0, Math.min(24 * 60 - 1, Math.floor(y)));
    const mins = Math.floor(clamped / 15) * 15;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
  }

  function startDragCreate(e: PointerEvent, dateStr: string) {
    // Ignore right-clicks and drags that begin on an event card
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest('[data-event-card]')) return;

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dragCreate = {
      dateStr,
      startY: e.clientY - rect.top,
      currentY: e.clientY - rect.top,
      active: true,
    };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function updateDragCreate(e: PointerEvent) {
    if (!dragCreate?.active) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dragCreate.currentY = e.clientY - rect.top;
  }

  function endDragCreate(e: PointerEvent) {
    if (!dragCreate?.active) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const endY = e.clientY - rect.top;

    const startTime = yToTime(dragCreate.startY);
    const endTime = yToTime(endY);

    let [sh, sm] = startTime.split(':').map(Number);
    let [eh, em] = endTime.split(':').map(Number);
    const startMins = sh * 60 + sm;
    const endMins = eh * 60 + em;

    // Enforce a minimum 30-minute span; ignore tiny accidental drags
    if (Math.abs(endMins - startMins) < 30) {
      dragCreate = null;
      return;
    }

    const [startStr, endStr] =
      endMins > startMins ? [startTime, endTime] : [endTime, startTime];

    // Only create if this wasn't a click (moved at least 30 minutes worth)
    suppressNextClick = true;
    onEmptySlotClick?.(dragCreate.dateStr, startStr);
    const customEv = new CustomEvent('emptySlotClickWithEndTime', {
      detail: { date: dragCreate.dateStr, startTime: startStr, endTime: endStr },
    });
    window.dispatchEvent(customEv);

    dragCreate = null;
  }

  // Derived ghost-rect style while dragging to create
  function getDragCreateStyle(): string {
    if (!dragCreate?.active) return '';
    const start = dragCreate.startY;
    const end = dragCreate.currentY;
    const top = Math.min(start, end);
    const height = Math.abs(end - start);
    return `top: ${top}px; height: ${Math.max(height, 2)}px;`;
  }

  // ── Drag-to-resize state & helpers ──────────────────────────────
  let resizing = $state<{
    id: string;
    startY: number;
    originalEndMins: number;
  } | null>(null);
  let resizePreviewEnd = $state<string | null>(null);

  function startResize(e: PointerEvent, ev: CalendarEvent) {
    e.preventDefault();
    e.stopPropagation();
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const [eh, em] = ev.endTime.split(':').map(Number);
    resizing = {
      id: ev.id,
      startY: e.clientY - rect.top,
      originalEndMins: eh * 60 + em,
    };
    resizePreviewEnd = ev.endTime;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function updateResize(e: PointerEvent) {
    if (!resizing) return;
    const el = e.currentTarget as HTMLElement | null;
    if (!el) return;
    const deltaMins = Math.round((e.clientY - el.getBoundingClientRect().top - resizing.startY) / 15) * 15;
    const newEndMins = Math.max(resizing.originalEndMins + deltaMins, 30);
    const h = Math.floor(newEndMins / 60);
    const m = newEndMins % 60;
    resizePreviewEnd = `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
  }

  function endResize(e: PointerEvent) {
    if (!resizing) return;
    updateResize(e);
    if (resizePreviewEnd && resizePreviewEnd !== resizing.originalEndMins.toString()) {
      const id = resizing.id;
      const newEnd = resizePreviewEnd;
      // Keep the new end clamped to the same day (never beyond 23:59)
      const [h, m] = newEnd.split(':').map(Number);
      const capped = h > 23 ? '23:59' : newEnd;
      onEventUpdate(id, { endTime: capped });
    }
    resizing = null;
    resizePreviewEnd = null;
  }

  function pointerCaptureLost() {
    dragCreate = null;
    resizing = null;
    resizePreviewEnd = null;
  }

  import { onMount } from 'svelte';
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Hover Popover State
  let hoveredEvent = $state<CalendarEvent | null>(null);
  let hoverAnchorElement = $state<HTMLElement | null>(null);
  let hoverTimer: ReturnType<typeof setTimeout> | null = null;
  let isHoveringPopover = $state(false);

  function handlePointerEnter(e: PointerEvent, ev: CalendarEvent) {
    if (resizing || dragCreate?.active || ev.id === selectedEventId) return;
    const target = e.currentTarget as HTMLElement;
    if (hoverTimer) clearTimeout(hoverTimer);

    hoverTimer = setTimeout(() => {
      hoveredEvent = ev;
      hoverAnchorElement = target;
    }, 250);
  }

  function handlePointerLeave(e: PointerEvent) {
    if (hoverTimer) clearTimeout(hoverTimer);
    hoverTimer = setTimeout(() => {
      if (!isHoveringPopover) {
        hoveredEvent = null;
        hoverAnchorElement = null;
      }
    }, 150);
  }

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
                in:scale={{ duration: 200, start: 0.95 }}
                onclick={(e) => {
                  hoveredEvent = null;
                  hoverAnchorElement = null;
                  onEventClick(ev, e);
                }}
                onpointerenter={(e) => handlePointerEnter(e, ev)}
                onpointerleave={handlePointerLeave}
                class="w-full px-2 py-0.5 rounded text-[10px] text-left font-medium truncate shadow-sm cursor-pointer transition-all hover:scale-[1.03] hover:shadow-lg hover:z-20 {(COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue).bg} {ev.id === selectedEventId ? 'ring-2 ring-white ring-offset-2 ring-offset-[#131313] z-10' : 'border-transparent'}"
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
         style="grid-template-columns: {gutterWidth}px repeat({visibleDates().length}, minmax(0, 1fr));">
      <div class="border-r border-[var(--color-border-hairline)] flex items-center justify-center">
        <div class="flex items-center w-full h-full divide-x divide-[var(--color-border-hairline)]">
          {#each secondaryTimezones as tz}
            <div class="flex-1 h-full p-2 flex flex-col items-center justify-center">
              <span class="text-[10px] opacity-70 truncate w-full text-center" title={tz}>{tz.split('/')[1]?.replace('_', ' ') || tz}</span>
            </div>
          {/each}
          <div class="flex-1 h-full p-2 flex flex-col items-center justify-center font-mono text-[10px]">
            <span class="opacity-70">{tzOffsetStr()}</span>
          </div>
        </div>
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
           style="grid-template-columns: {gutterWidth}px repeat({visibleDates().length}, minmax(0, 1fr));">
        
        <!-- Y-Axis Hours list labels -->
        <div class="border-r border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)]/10 flex divide-x divide-[var(--color-border-hairline)]">
          {#each secondaryTimezones as tz}
            <div class="flex-1 flex flex-col">
              {#each hours as hour}
                <div class="h-[60px] p-2 text-[10px] font-mono text-[var(--color-text-secondary)]/40 text-right flex items-start justify-end">
                  {formatHourInTimezone(hour, tz)}
                </div>
              {/each}
            </div>
          {/each}
          <div class="flex-[1.1] flex flex-col">
            {#each hours as hour}
              <div class="h-[60px] p-2 text-[10px] font-mono text-[var(--color-text-secondary)]/60 text-right">
                {hour === 0 ? '12 AM' : hour === 12 ? '12 PM' : hour > 12 ? `${hour - 12} PM` : `${hour} AM`}
              </div>
            {/each}
          </div>
        </div>

        <!-- Columns for each day -->
        {#each visibleDates() as date}
          {@const dateStr = toISODateString(date)}
          {@const dayEvents = getEventsForDate(dateStr)}
          {@const layoutInfo = getTimedEventsLayout(dayEvents)}
          <div class="border-r border-[var(--color-border-hairline)]/30 last:border-r-0 relative hover:bg-[var(--color-canvas-hover)]/5 transition-colors cursor-cell select-none"
               ondragover={(e) => { e.preventDefault(); e.dataTransfer!.dropEffect = 'move'; }}
               ondrop={(e) => {
                 e.preventDefault();
                 const id = e.dataTransfer?.getData('text/plain');
                 if (!id) return;
                 const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                 const y = e.clientY - rect.top;
                 const exactHour = Math.floor(y / 60);
                 const roundedMinutes = Math.floor((y % 60) / 15) * 15;
                 const newStartTime = `${exactHour.toString().padStart(2, '0')}:${roundedMinutes.toString().padStart(2, '0')}`;
                 
                 // Get old event to calculate duration and new end time
                 const ev = events.find((e: any) => e.id === id);
                 if (ev) {
                   const [sh, sm] = ev.startTime.split(':').map(Number);
                   const [eh, em] = ev.endTime.split(':').map(Number);
                   const duration = (eh * 60 + em) - (sh * 60 + sm);
                   
                   const newStartMins = exactHour * 60 + roundedMinutes;
                   const newEndMins = newStartMins + duration;
                   const newEndHour = Math.floor(newEndMins / 60);
                   const newEndMinute = newEndMins % 60;
                   const newEndTime = `${newEndHour.toString().padStart(2, '0')}:${newEndMinute.toString().padStart(2, '0')}`;
                   
                   onEventUpdate(id, {
                     date: dateStr,
                     startTime: newStartTime,
                     endTime: newEndTime
                   });
                 }
               }}
               onpointerdown={(e) => startDragCreate(e, dateStr)}
               onpointermove={(e) => updateDragCreate(e)}
               onpointerup={(e) => endDragCreate(e)}
               onpointercancel={pointerCaptureLost}
               onclick={(e) => {
                 // A drag-to-create that just committed will fire this click too — skip it.
                 if (suppressNextClick) {
                   suppressNextClick = false;
                   return;
                 }
                 // Fractional time selection based on exact click coordinate
                 const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
                 const y = e.clientY - rect.top;
                 
                 // 1 hour = 60px height
                 const hoursDecimal = y / 60;
                 const exactHour = Math.floor(hoursDecimal);
                 const exactMinutes = Math.floor((hoursDecimal - exactHour) * 60);
                 const roundedMinutes = Math.floor(exactMinutes / 15) * 15; // round to nearest 15
                 
                 // End time is 1 hour later, capped at 23:59
                 const endHour = Math.min(23, exactHour + 1);
                 const endMinutes = exactHour === 23 && roundedMinutes > 0 ? 59 : roundedMinutes;
                 
                 const startTimeStr = `${exactHour.toString().padStart(2, '0')}:${roundedMinutes.toString().padStart(2, '0')}`;
                 const endTimeStr = `${endHour.toString().padStart(2, '0')}:${endMinutes.toString().padStart(2, '0')}`;
                 
                 // We add a third parameter to onEmptySlotClick in +page.svelte (endTimeStr)
                 onEmptySlotClick?.(dateStr, startTimeStr, e as any);
                 // We will temporarily pass endTimeStr via a custom event since we can't easily change the prop signature everywhere without looking.
                 const customEv = new CustomEvent('emptySlotClickWithEndTime', { detail: { date: dateStr, startTime: startTimeStr, endTime: endTimeStr }});
                 window.dispatchEvent(customEv);
               }}
               role="button"
               tabindex="0"
               onkeydown={(e) => { if (e.key === 'Enter') onEmptySlotClick?.(dateStr, '09:00'); }}
          >
            <!-- Drag-to-create ghost selection overlay -->
            {#if dragCreate?.active && dragCreate.dateStr === dateStr}
              <div
                class="absolute left-0 right-0 bg-blue-500/25 border border-blue-400/50 rounded-md pointer-events-none z-20"
                style={getDragCreateStyle()}
              ></div>
            {/if}

            <!-- Render events for this column day -->
            {#each dayEvents as ev}
              {@const top = getEventTopOffset(ev.startTime)}
              {@const height = getEventHeight(ev.startTime, ev.endTime)}
              {@const colStyle = COLOR_CLASSES[ev.color] || COLOR_CLASSES.blue}
              {@const layout = layoutInfo.get(ev.id)}
              {@const widthPct = layout ? layout.width : 100}
              {@const leftPct = layout ? layout.left : 0}
              {@const isSelected = ev.id === selectedEventId}
              {@const previewEnd = resizing?.id === ev.id ? resizePreviewEnd : ev.endTime}
              {@const displayHeight = resizing?.id === ev.id ? getEventHeight(ev.startTime, previewEnd ?? ev.endTime) : height}
              
              <button
                in:scale={{ duration: 200, start: 0.95 }}
                draggable="true"
                ondragstart={(e) => {
                  e.dataTransfer!.setData('text/plain', ev.id);
                  e.dataTransfer!.effectAllowed = 'move';
                }}
                onclick={(e) => {
                  e.stopPropagation();
                  hoveredEvent = null;
                  hoverAnchorElement = null;
                  onEventClick(ev, e);
                }}
                onpointerenter={(e) => handlePointerEnter(e, ev)}
                onpointerleave={handlePointerLeave}
                data-event-card
                class="absolute p-2 rounded-lg text-xs text-left font-medium border shadow-md transition-all hover:scale-[1.03] hover:shadow-lg cursor-move overflow-hidden
                  {colStyle.bg} {colStyle.border} {isSelected ? 'ring-2 ring-white ring-offset-2 ring-offset-[#131313] z-50' : 'hover:z-50 z-10'}"
                style="top: {top}px; height: {displayHeight}px; left: {leftPct}%; width: calc({widthPct}% - 4px);"
              >
                <div class="font-bold truncate text-white leading-tight pointer-events-none">{ev.title}</div>
                <div class="text-[10px] opacity-75 font-mono mt-0.5 pointer-events-none">
                  {ev.startTime} - {resizing?.id === ev.id ? (resizePreviewEnd ?? ev.endTime) : ev.endTime}
                </div>
                {#if ev.location && height > 50}
                  <div class="text-[9px] opacity-80 truncate flex items-center gap-1 mt-1 pointer-events-none">
                    <MapPin class="w-3 h-3 text-current shrink-0" />
                    <span class="truncate">{ev.location}</span>
                  </div>
                {/if}

                <!-- Drag-to-resize handle (bottom edge) -->
                {#if isSelected}
                  <div
                    class="absolute bottom-0 left-0 right-0 h-1.5 cursor-ns-resize bg-white/20 hover:bg-white/50 rounded-b-lg transition-colors"
                    onpointerdown={(e) => startResize(e, ev)}
                    onpointermove={(e) => updateResize(e)}
                    onpointerup={(e) => endResize(e)}
                    onpointercancel={pointerCaptureLost}
                  ></div>
                {/if}
              </button>
            {/each}
          </div>
        {/each}

      </div>
    </div>
  {/if}
</div>

<EventHoverPopover
  event={hoveredEvent}
  anchor={hoverAnchorElement}
  onMouseEnter={() => {
    isHoveringPopover = true;
    if (hoverTimer) clearTimeout(hoverTimer);
  }}
  onMouseLeave={() => {
    isHoveringPopover = false;
    hoveredEvent = null;
    hoverAnchorElement = null;
  }}
/>
