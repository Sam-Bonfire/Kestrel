<script lang="ts">
  import CalendarSidebar, { type Account, type Calendar } from '$lib/components/CalendarSidebar.svelte';
  import WeekGrid, { type CalendarEvent } from '$lib/components/WeekGrid.svelte';
  import EventPeekPanel from '$lib/components/EventPeekPanel.svelte';
  import EventModal from '$lib/components/EventModal.svelte';
  import { 
    Calendar as CalendarIcon, ChevronLeft, ChevronRight, Grid, List, Clock, AlignLeft 
  } from 'lucide-svelte';
  import { WindowControls } from '@kestrel/shared/components';
  import { Login } from '@kestrel/shared/components';
  import { authState } from '@kestrel/shared/stores';

  // State management
  let selectedDate = $state(new Date());
  let viewMode = $state<'month' | 'week' | 'day' | 'weekdays' | 'agenda'>('month');
  let isNewEventModalOpen = $state(false);
  let selectedEvent = $state<any | null>(null);
  let clickPosition = $state<{x: number, y: number} | null>(null);

  // Swipe logic state
  let touchStartX = $state(0);
  let touchEndX = $state(0);
  
  // Mobile responsive state
  let isMobileOrTablet = $state(false);
  
  import { onMount } from 'svelte';
  onMount(() => {
    const checkMobile = () => isMobileOrTablet = window.innerWidth <= 768;
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  });

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
  }
  
  function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].clientX;
    handleSwipe();
  }
  
  function handleSwipe() {
    const threshold = 50; // minimum distance to be considered a swipe
    if (touchEndX < touchStartX - threshold) handleNavigateDate('next'); // swipe left
    if (touchEndX > touchStartX + threshold) handleNavigateDate('prev'); // swipe right
  }

  // Dynamic calendars metadata structure (Matching prototype)
  let accounts = $state<Account[]>([
    {
      id: '1',
      email: 'alex@kestrel.dev',
      isExpanded: true,
      calendars: [
        { id: '1a', name: 'Workspace Tasks', color: 'blue', isActive: true, isDefault: true },
        { id: '1b', name: 'Engineering Syncs', color: 'purple', isActive: true }
      ]
    },
    {
      id: '2',
      email: 'alex@gmail.com',
      isExpanded: true,
      calendars: [
        { id: '2a', name: 'Personal Schedule', color: 'green', isActive: true },
        { id: '2b', name: 'Holidays & Reminders', color: 'orange', isActive: false }
      ]
    }
  ]);

  // Calendar Events (Populated dynamically)
  let events = $state<CalendarEvent[]>([
    {
      id: '1',
      title: 'Workspace Design Sync',
      date: new Date().toISOString().split('T')[0],
      startTime: '10:00',
      endTime: '11:30',
      color: 'blue',
      location: 'Gather Town Workspace',
      description: 'Review frontend Svelte tokens and design mappings with team.',
      calendarId: '1a',
      category: 'Work',
      priority: 'High',
      status: 'Scheduled'
    },
    {
      id: '2',
      title: 'Engineering Review: Axum & SQLx',
      date: new Date(Date.now() + 86400000).toISOString().split('T')[0], // tomorrow
      startTime: '13:00',
      endTime: '14:30',
      color: 'purple',
      location: 'https://zoom.us/j/kestrel-sync',
      description: 'Discuss SQLite offline queue sync loops and telemetry logger.',
      calendarId: '1b',
      category: 'Workspace',
      priority: 'Medium',
      status: 'Scheduled'
    },
    {
      id: '3',
      title: 'Weekend Yoga Class',
      date: new Date(Date.now() + 86400000 * 2).toISOString().split('T')[0], // 2 days from now
      startTime: '09:00',
      endTime: '10:00',
      color: 'green',
      location: 'Kestrel Wellness Center',
      description: 'Morning alignment and breathing.',
      calendarId: '2a',
      category: 'Personal',
      priority: 'None',
      status: 'Scheduled'
    }
  ]);

  // Derived filtered events matching only ACTIVE/CHECKED calendars
  let activeCalendarIds = $derived(
    accounts.flatMap(acc => acc.calendars.filter(cal => cal.isActive).map(cal => cal.id))
  );

  let filteredEvents = $derived(
    events.filter(e => activeCalendarIds.includes(e.calendarId))
  );

  function handleToggleCalendar(accId: string, calId: string) {
    accounts = accounts.map(acc => {
      if (acc.id === accId) {
        return {
          ...acc,
          calendars: acc.calendars.map(cal => cal.id === calId ? { ...cal, isActive: !cal.isActive } : cal)
        };
      }
      return acc;
    });
  }

  function handleSaveEvent(data: any) {
    const newEvent: CalendarEvent = {
      id: String(events.length + 1),
      title: data.title,
      description: data.description,
      location: data.location,
      date: data.date,
      startTime: data.startTime,
      endTime: data.endTime,
      color: data.color,
      calendarId: accounts[0].calendars[0].id, // Default to primary calendar
      category: data.category,
      priority: data.priority,
      status: data.status,
      rsvpStatus: data.rsvpStatus
    };
    events = [...events, newEvent];
  }

  // Navigate Date
  function handleNavigateDate(direction: 'prev' | 'next') {
    const d = new Date(selectedDate);
    const step = viewMode === 'day' ? 1 : viewMode === 'week' ? 7 : viewMode === 'weekdays' ? 7 : 30;
    d.setDate(d.getDate() + (direction === 'prev' ? -step : step));
    selectedDate = d;
  }

  function handleJumpToToday() {
    selectedDate = new Date();
  }
</script>

{#if !authState.isAuthenticated}
  <Login />
{:else}
<div class="flex h-screen w-screen overflow-hidden bg-[var(--color-canvas-base)]">
  <!-- Calendar Sidebar navigation -->
  {#if !isMobileOrTablet}
    <CalendarSidebar
      {selectedDate}
      onDateSelect={(d) => selectedDate = d}
      onNewEventClick={() => isNewEventModalOpen = true}
      bind:accounts
      onToggleCalendar={handleToggleCalendar}
    />
  {/if}

  <!-- Main View Canvas area -->
  <div class="flex-1 flex flex-col overflow-hidden" 
       ontouchstart={handleTouchStart} 
       ontouchend={handleTouchEnd}
       role="region" aria-label="Calendar Canvas">
    
    <!-- Top toolbar header -->
    <header 
      class="px-6 py-2 flex items-center justify-between shrink-0 bg-[var(--color-canvas-base)] cursor-default"
      onpointerdown={(e) => {
        if (e.target === e.currentTarget) {
          import('@tauri-apps/api/window').then(m => m.getCurrentWindow().startDragging());
        }
      }}
    >
      <div class="flex items-center gap-4">
        <!-- Date display header -->
        <h1 class="text-sm font-bold text-white leading-none">
          {selectedDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}
        </h1>

        <!-- Back/Next navigation -->
        <div class="flex items-center gap-1">
          <button 
            onclick={() => handleNavigateDate('prev')}
            class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <button 
            onclick={handleJumpToToday}
            class="px-2.5 py-1 text-[11px] font-semibold border border-[var(--color-border-hairline)] hover:bg-[var(--color-canvas-hover)] rounded-md text-white transition-colors cursor-pointer"
          >
            Today
          </button>
          <button 
            onclick={() => handleNavigateDate('next')}
            class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- View Selector Tabs (Month, Week, Day, Agenda) -->
      {#if !isMobileOrTablet}
        <div class="flex items-center gap-1 text-xs">
          {#each [
            { mode: 'month', label: 'Month', icon: CalendarIcon },
            { mode: 'week', label: 'Week', icon: Grid },
            { mode: 'day', label: 'Day', icon: Clock },
            { mode: 'agenda', label: 'Agenda', icon: AlignLeft }
          ] as tab}
            <button
              onclick={() => viewMode = tab.mode as any}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all cursor-pointer font-medium
                {viewMode === tab.mode 
                  ? 'bg-white text-black font-semibold shadow' 
                  : 'border-transparent text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]'}"
            >
              <tab.icon class="w-3.5 h-3.5" />
              <span>{tab.label}</span>
            </button>
          {/each}
        </div>
      {:else}
        <!-- Mobile View Toggle (Simplified) -->
        <select bind:value={viewMode} class="bg-[#2A2A2A] text-white text-xs p-1.5 rounded-md border border-[var(--color-border-hairline)] outline-none">
          <option value="month">Month</option>
          <option value="week">Week</option>
          <option value="day">Day</option>
          <option value="agenda">Agenda</option>
        </select>
      {/if}

      <!-- Desktop window controls (since app is borderless) -->
      <div class="hidden sm:block ml-2 border-l border-[var(--color-border-hairline)] pl-2 h-6 flex items-center">
        <WindowControls />
      </div>
    </header>

    <!-- Unified timeline/month/agenda grid view component -->
    <WeekGrid
      events={filteredEvents}
      {selectedDate}
      bind:viewMode
      onChangeViewMode={(v) => viewMode = v}
      onEventClick={(ev, e) => {
        if (e) {
          clickPosition = { x: e.clientX, y: e.clientY };
        }
        selectedEvent = ev;
      }}
      onEmptySlotClick={(dateStr, timeStr) => {
        selectedDate = new Date(dateStr);
        // We'd ideally prepopulate the time too, but currently EventModal takes selectedDateStr
        isNewEventModalOpen = true;
      }}
    />
  </div>

  <!-- Event Details Sidebar Peek -->
  {#if selectedEvent}
    <EventPeekPanel
      event={selectedEvent}
      {clickPosition}
      onClose={() => selectedEvent = null}
      onEdit={() => { selectedEvent = null; isNewEventModalOpen = true; }}
      onDelete={() => {
        events = events.filter(e => e.id !== selectedEvent.id);
        selectedEvent = null;
      }}
    />
  {/if}

  <!-- Creation & Editing Dialog Form Modal -->
  <EventModal
    isOpen={isNewEventModalOpen}
    selectedDateStr={selectedDate.toISOString().split('T')[0]}
    onClose={() => isNewEventModalOpen = false}
    onSave={handleSaveEvent}
  />
</div>
{/if}
