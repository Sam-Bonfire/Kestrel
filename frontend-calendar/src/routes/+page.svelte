<script lang="ts">
  import CalendarSidebar, { type Account, type Calendar } from '$lib/components/CalendarSidebar.svelte';
  import WeekGrid, { type CalendarEvent } from '$lib/components/WeekGrid.svelte';
  import EventPeekPanel from '$lib/components/EventPeekPanel.svelte';
  import { 
    Calendar as CalendarIcon, ChevronLeft, ChevronRight, Grid, List, Clock, AlignLeft,
    Search, Settings, Menu, ChevronDown, X, CalendarDays
  } from 'lucide-svelte';
  import { AppShell } from '@kestrel/shared/components';
  import { authState } from '@kestrel/shared/stores';

  // State management
  $effect(() => {
    if (authState.isInitialized && !authState.isAuthenticated) {
      import('$app/navigation').then(({ goto }) => goto('/login'));
    }
  });

  let selectedDate = $state(new Date());
  let viewMode = $state<string>('month');
  let selectedEvent = $state<any | null>(null);
  let clickPosition = $state<{x: number, y: number} | null>(null);

  // Swipe logic state
  let touchStartX = $state(0);
  let touchStartY = $state(0);
  let touchEndX = $state(0);
  let touchEndY = $state(0);
  
  // Toolbar state
  let searchQuery = $state('');
  let isMobileSearchOpen = $state(false);
  let isViewDropdownOpen = $state(false);
  let dropdownSubmenu = $state<'none' | 'number_of_days' | 'settings'>('none');
  let isSettingsOpen = $state(false);
  let defaultCalendarId = $state('cal-personal');
  let startHour = $state(8);
  let showWeekends = $state(true);
  let isHeaderMonthDropdownOpen = $state(false);
  let miniMonth = $state(new Date());
  let isDetailsDocked = $state(false);

  // LocalStorage state persistence
  $effect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('kestrel_accounts', JSON.stringify(accounts));
      localStorage.setItem('kestrel_events', JSON.stringify(events));
      localStorage.setItem('kestrel_viewMode', viewMode);
      localStorage.setItem('kestrel_showWeekends', JSON.stringify(showWeekends));
      localStorage.setItem('kestrel_isDocked', JSON.stringify(isDetailsDocked));
      localStorage.setItem('kestrel_startHour', startHour.toString());
    }
  });

  // Toast state
  let toastMessage = $state<{ text: string, type: 'success'|'info'|'error', id: number } | null>(null);
  let toastTimeout: ReturnType<typeof setTimeout>;

  function showToast(text: string, type: 'success'|'info'|'error' = 'info') {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastMessage = { text, type, id: Date.now() };
    toastTimeout = setTimeout(() => {
      toastMessage = null;
    }, 3000);
  }

  function openNewEventPanel(dateStr: string, timeStr = '10:00', e?: MouseEvent) {
    selectedDate = new Date(dateStr);
    if (e) {
      clickPosition = { x: e.clientX, y: e.clientY };
    } else {
      clickPosition = null;
    }
    
    // Calculate end time (1 hour later by default)
    let [hours, minutes] = timeStr.split(':').map(Number);
    hours = (hours + 1) % 24;
    const endStr = `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
    
    selectedEvent = {
      title: '',
      date: dateStr,
      startTime: timeStr,
      endTime: endStr,
      color: 'blue',
      category: 'Work',
      status: 'Scheduled',
      priority: 'None'
    };
  }

  // Keyboard shortcuts
  function handleGlobalKeydown(e: KeyboardEvent) {
    // Don't trigger if user is typing in an input or contentEditable element
    const target = e.target as HTMLElement;
    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target.isContentEditable) return;

    const key = e.key.toLowerCase();
    
    // View switching
    if (key === 'd') viewMode = 'day';
    else if (key === 'w') viewMode = 'week';
    else if (key === 'm') viewMode = 'month';
    else if (key === 'a') viewMode = 'agenda';
    else if (key === 't') handleJumpToToday();
    // N-Day views (1-7)
    else if (['1','2','3','4','5','6','7'].includes(key)) {
      if (key === '1') viewMode = 'day';
      else if (key === '7') viewMode = 'week';
      else viewMode = `${key}-day`;
      showToast(`${key}-day view activated`, 'info');
    }
  }

  // Mobile responsive state
  let isMobileOrTablet = $state(false);
  let isSidebarOpenMobile = $state(false);
  
  import { onMount } from 'svelte';
  import { initAuth } from '@kestrel/shared/stores';

  onMount(() => {
    initAuth();
    // Load state from localStorage
    try {
      const savedAccounts = localStorage.getItem('kestrel_accounts');
      if (savedAccounts) accounts = JSON.parse(savedAccounts);
      const savedEvents = localStorage.getItem('kestrel_events');
      if (savedEvents) events = JSON.parse(savedEvents);
      const savedViewMode = localStorage.getItem('kestrel_viewMode');
      if (savedViewMode) viewMode = savedViewMode;
      const savedShowWeekends = localStorage.getItem('kestrel_showWeekends');
      if (savedShowWeekends) showWeekends = JSON.parse(savedShowWeekends);
      const savedDocked = localStorage.getItem('kestrel_isDocked');
      if (savedDocked) isDetailsDocked = JSON.parse(savedDocked);
      const savedStartHour = localStorage.getItem('kestrel_startHour');
      if (savedStartHour) startHour = parseInt(savedStartHour, 10);
    } catch(e) {}

    const checkMobile = () => isMobileOrTablet = window.innerWidth <= 1024;
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  });

  $effect(() => {
    if (authState.isAuthenticated) {
      import('@kestrel/shared/api').then(({ getEvents }) => {
        import('rrule').then(({ RRule }) => {
          // Fetch events for the current month roughly
          const now = new Date();
          const start = new Date(now.getFullYear(), now.getMonth() - 1, 1).toISOString();
          const end = new Date(now.getFullYear(), now.getMonth() + 2, 1).toISOString();
          getEvents(start, end).then(res => {
            if (res && res.events && res.events.length > 0) {
              const expandedEvents: any[] = [];
              res.events.forEach((e: any) => {
                const baseEvent = {
                  id: e.id,
                  title: e.title || 'Untitled Event',
                  date: new Date(e.start_time * 1000).toISOString().split('T')[0],
                  startTime: new Date(e.start_time * 1000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }),
                  endTime: new Date(e.end_time * 1000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }),
                  color: 'blue',
                  location: e.location,
                  description: e.description,
                  calendarId: accounts[0]?.calendars[0]?.id || '1a',
                  category: 'Work',
                  priority: 'None',
                  status: 'Scheduled',
                  organizer: e.organizer_email || 'Unknown',
                  attendees: e.attendees ? JSON.parse(e.attendees) : [],
                  rsvpStatus: 'maybe'
                };

                if (e.rrule) {
                  try {
                    const rule = RRule.fromString(e.rrule);
                    const occurrences = rule.between(new Date(start), new Date(end), true);
                    occurrences.forEach((occ: Date, index: number) => {
                      expandedEvents.push({
                        ...baseEvent,
                        id: `${e.id}-${index}`, // unique ID for the instance
                        date: occ.toISOString().split('T')[0]
                      });
                    });
                  } catch (err) {
                    console.error('Failed to parse rrule for event:', e.id, err);
                    expandedEvents.push(baseEvent);
                  }
                } else {
                  expandedEvents.push(baseEvent);
                }
              });
              events = expandedEvents;
            }
          }).catch(console.error);
        });
      });
    }
  });

  let snoozedEvents = $state<Record<string, number>>({});

  $effect(() => {
    if (authState.isAuthenticated) {
      import('@tauri-apps/plugin-notification').then(({ sendNotification, onAction }) => {
        
        onAction((event) => {
          if (event.actionId === 'snooze' && event.notification.id) {
            // Snooze for 10 minutes
            snoozedEvents[event.notification.id] = Date.now() + 10 * 60 * 1000;
          } else if (event.actionId === 'dismiss' && event.notification.id) {
            // Dismiss permanently
            snoozedEvents[event.notification.id] = Number.MAX_SAFE_INTEGER;
          }
        }).catch(err => console.warn("Notification action listener error:", err));

        const checkUpcomingEvents = () => {
          const now = new Date();
          events.forEach(ev => {
            const eventTime = new Date(`${ev.date}T${ev.startTime}:00`).getTime();
            const timeUntil = eventTime - now.getTime();
            
            // Notify if event is exactly 10 minutes away, or if it was snoozed and the snooze time is up
            const is10MinWarning = timeUntil > 9 * 60 * 1000 && timeUntil <= 10 * 60 * 1000;
            const isSnoozeUp = snoozedEvents[ev.id] && now.getTime() > snoozedEvents[ev.id] && snoozedEvents[ev.id] !== Number.MAX_SAFE_INTEGER;
            
            if (is10MinWarning || isSnoozeUp) {
              sendNotification({
                id: ev.id,
                title: `Upcoming: ${ev.title}`,
                body: `Starts at ${ev.startTime} ${ev.location ? `in ${ev.location}` : ''}`,
              });
              if (isSnoozeUp) {
                delete snoozedEvents[ev.id]; // clear snooze
              } else {
                snoozedEvents[ev.id] = Number.MAX_SAFE_INTEGER; // prevent duplicate 10m warnings
              }
            }
          });
        };

        const interval = setInterval(checkUpcomingEvents, 60000);
        return () => clearInterval(interval);
      }).catch(() => {});
    }
  });

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].clientX;
    touchStartY = e.changedTouches[0].clientY;
  }
  
  function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].clientX;
    touchEndY = e.changedTouches[0].clientY;
    handleSwipe();
  }
  
  function handleSwipe() {
    const deltaX = touchEndX - touchStartX;
    const deltaY = touchEndY - touchStartY;
    
    // Swipe Navigation Vector Math (Task 27)
    if (Math.abs(deltaX) > Math.abs(deltaY) && Math.abs(deltaX) > 50) {
      if (deltaX < 0) handleNavigateDate('next');
      else handleNavigateDate('prev');
    }
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
      status: 'Scheduled',
      organizer: 'Sam (You)',
      rsvpStatus: 'yes',
      attendees: [
        { name: 'Alex Rivera', email: 'arivera@kestrel.inc', rsvp: 'yes' },
        { name: 'Jordan Lee', email: 'jlee@kestrel.inc', rsvp: 'maybe' }
      ]
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
      status: 'Scheduled',
      organizer: 'Backend Guild',
      rsvpStatus: 'maybe',
      attendees: [
        { name: 'Casey Smith', email: 'csmith@kestrel.inc', rsvp: 'yes' },
        { name: 'Sam (You)', email: 'sam@kestrel.inc', rsvp: 'maybe' }
      ]
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

  let activeCalendarIds = $derived(
    accounts.flatMap(acc => acc.calendars.filter(cal => cal.isActive).map(cal => cal.id))
  );

  let filteredEvents = $derived(
    events.filter(e => activeCalendarIds.includes(e.calendarId))
  );

  let headerLabel = $derived.by(() => {
    if (viewMode === 'day') return selectedDate.toLocaleDateString(undefined, { month: 'long', day: 'numeric', year: 'numeric' });
    if (viewMode === 'month') return selectedDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
    
    // For week/multi-day views, we show a range
    let daysToAdd = 6;
    if (viewMode === 'weekdays') daysToAdd = 4;
    const nDayMatch = viewMode.match(/^(\d+)-day$/);
    if (nDayMatch) daysToAdd = parseInt(nDayMatch[1], 10) - 1;

    let start = new Date(selectedDate);
    if (viewMode === 'week' || viewMode === 'weekdays') {
      const day = start.getDay();
      const diff = start.getDate() - day + (viewMode === 'weekdays' ? 1 : 0);
      start.setDate(diff);
    }
    
    const end = new Date(start);
    end.setDate(end.getDate() + daysToAdd);

    if (start.getMonth() === end.getMonth() && start.getFullYear() === end.getFullYear()) {
      return `${start.toLocaleDateString(undefined, { month: 'long' })} ${start.getDate()} – ${end.getDate()}, ${start.getFullYear()}`;
    } else if (start.getFullYear() === end.getFullYear()) {
      return `${start.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} – ${end.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}, ${start.getFullYear()}`;
    }
    return `${start.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })} – ${end.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}`;
  });

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

  let viewModeLabel = $derived.by(() => {
    if (viewMode === 'day') return 'Day';
    if (viewMode === 'week') return 'Week';
    if (viewMode === 'month') return 'Month';
    if (viewMode === 'agenda') return 'Agenda';
    if (viewMode === 'weekdays') return 'Weekdays';
    const nDayMatch = viewMode.match(/^(\d+)-day$/);
    if (nDayMatch) return `${nDayMatch[1]} days`;
    return viewMode;
  });

  let headerDaysInMonth = $derived.by(() => {
    const year = miniMonth.getFullYear();
    const month = miniMonth.getMonth();
    const firstDay = new Date(year, month, 1).getDay();
    const daysCount = new Date(year, month + 1, 0).getDate();
    
    const days: (Date | null)[] = [];
    for (let i = 0; i < firstDay; i++) days.push(null);
    for (let d = 1; d <= daysCount; d++) days.push(new Date(year, month, d));
    return days;
  });

  function handleSaveEvent(data: any) {
    import('@kestrel/shared/api').then(({ createEvent }) => {
      // Map form data to backend payload
      const startDateTime = new Date(`${data.date}T${data.startTime}:00`);
      const endDateTime = new Date(`${data.date}T${data.endTime}:00`);
      
      const payload = {
        title: data.title || 'Untitled Event',
        description: data.description,
        location: data.location,
        start_time: Math.floor(startDateTime.getTime() / 1000),
        end_time: Math.floor(endDateTime.getTime() / 1000),
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        is_all_day: data.isAllDay || false,
        organizer_email: data.organizer,
        attendees: data.attendees ? JSON.stringify(data.attendees) : null
      };

      createEvent(payload as any).then((res: any) => {
        const newEvent: CalendarEvent = {
          id: res.id,
          title: res.title || 'Untitled Event',
          description: res.description,
          location: res.location,
          date: new Date(res.start_time * 1000).toISOString().split('T')[0],
          startTime: new Date(res.start_time * 1000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }),
          endTime: new Date(res.end_time * 1000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }),
          isAllDay: res.is_all_day,
          color: data.color || 'blue',
          calendarId: data.calendarId || accounts[0]?.calendars[0]?.id || '1a',
          category: data.category,
          priority: data.priority,
          status: 'Scheduled',
          organizer: res.organizer_email || 'Unknown',
          attendees: res.attendees ? JSON.parse(res.attendees) : [],
          rsvpStatus: 'maybe'
        };
        events = [...events, newEvent];
        showToast('Event created successfully', 'success');
      }).catch(err => {
        console.error('Failed to create event:', err);
        showToast('Failed to create event', 'error');
      });
    });
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

<AppShell bind:isMobileSidebarOpen={isSidebarOpenMobile}>
  {#snippet sidebar()}
    <CalendarSidebar
      {selectedDate}
      onDateSelect={(d) => { selectedDate = d; if (isMobileOrTablet) isSidebarOpenMobile = false; }}
      onNewEventClick={() => {
        const now = new Date();
        now.setHours(now.getHours() + 1, 0, 0, 0);
        const hh = String(now.getHours()).padStart(2, '0');
        const mm = String(now.getMinutes()).padStart(2, '0');
        openNewEventPanel(selectedDate.toISOString().split('T')[0], `${hh}:${mm}`);
      }}
      bind:accounts
      onToggleCalendar={handleToggleCalendar}
      events={filteredEvents}
      {isMobileOrTablet}
      {viewMode}
      onViewModeChange={(mode) => {
        viewMode = mode;
        if (isMobileOrTablet) isSidebarOpenMobile = false;
      }}
    />
  {/snippet}

  {#snippet children()}
  <!-- Main View Canvas area -->
  <div class="flex-1 flex flex-col overflow-hidden transition-all duration-300 {isDetailsDocked && selectedEvent ? 'lg:mr-80' : ''}" 
       ontouchstart={handleTouchStart} 
       ontouchend={handleTouchEnd}
       role="region" aria-label="Calendar Canvas">
    
    {#if isMobileOrTablet}
      <!-- Mobile & Tablet Header -->
      <header class="pl-4 pr-36 py-3 border-b border-[var(--color-border-hairline)] flex items-center justify-between gap-2 bg-[#0a0a0a] relative select-none animate-fadeIn shrink-0">
        <!-- Transparent drag handle that stops before WindowControls -->
        <div class="absolute inset-y-0 left-0 right-36" data-tauri-drag-region></div>

        <!-- Left side: Hamburger, Month/Year + Dropdown Icon -->
        <div class="flex items-center gap-2 relative z-10">
          <button
            onclick={() => isSidebarOpenMobile = true}
            class="p-1.5 rounded-lg bg-[var(--color-canvas-card)] hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer border border-[var(--color-border-hairline)] flex items-center justify-center shrink-0"
            title="Open Sidebar"
          >
            <Menu class="w-4 h-4" />
          </button>

          <button
            onclick={() => isHeaderMonthDropdownOpen = !isHeaderMonthDropdownOpen}
            class="flex items-center gap-1 text-sm font-semibold text-white font-mono tracking-tight cursor-pointer hover:text-white p-1 rounded transition-colors"
          >
            <span>{selectedDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}</span>
            <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)] transition-transform duration-200 {isHeaderMonthDropdownOpen ? 'rotate-180' : ''}" />
          </button>
          
          {#if isHeaderMonthDropdownOpen}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="fixed inset-0 z-40" onclick={() => isHeaderMonthDropdownOpen = false} role="presentation"></div>
            <div class="fixed left-4 right-4 top-16 bg-[#131313] border border-neutral-800 rounded-xl shadow-2xl p-4 z-50 animate-fadeIn space-y-3 max-w-[320px] mx-auto">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs font-mono font-medium text-white">
                  {miniMonth.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}
                </span>
                <div class="flex gap-1">
                  <button class="p-1 rounded hover:bg-neutral-800 text-neutral-400" onclick={() => { const d = new Date(miniMonth); d.setMonth(d.getMonth() - 1); miniMonth = d; }}><ChevronLeft class="w-3 h-3" /></button>
                  <button class="p-1 rounded hover:bg-neutral-800 text-neutral-400" onclick={() => { const d = new Date(miniMonth); d.setMonth(d.getMonth() + 1); miniMonth = d; }}><ChevronRight class="w-3 h-3" /></button>
                </div>
              </div>
              <div class="grid grid-cols-7 gap-1 text-center text-[10px] font-mono text-[var(--color-text-secondary)]/50 font-medium mb-1.5">
                <span>Su</span><span>Mo</span><span>Tu</span><span>We</span><span>Th</span><span>Fr</span><span>Sa</span>
              </div>
              <div class="grid grid-cols-7 gap-1 text-center text-xs font-mono">
                {#each headerDaysInMonth as day}
                  {#if day}
                    {@const isSelected = day.toDateString() === selectedDate.toDateString()}
                    {@const isToday = day.toDateString() === new Date().toDateString()}
                    <button
                      onclick={() => { 
                        selectedDate = day!; 
                        isHeaderMonthDropdownOpen = false; 
                      }}
                      class="h-6 w-full rounded flex items-center justify-center transition-all cursor-pointer relative text-[11px]
                        {isToday 
                          ? 'bg-[var(--color-today-red)] text-white font-semibold shadow-sm hover:bg-[var(--color-today-red)]/90' 
                          : isSelected 
                          ? 'border border-white/20 bg-[var(--color-canvas-hover)] text-white font-semibold' 
                          : 'text-[var(--color-text-primary)] hover:bg-[var(--color-canvas-hover)]'}"
                    >
                      <span class={isToday || isSelected ? 'font-semibold' : ''}>{day.getDate()}</span>
                      {#if events.some(e => e.date === day.toISOString().split('T')[0])}
                        <span class="absolute bottom-0.5 w-1 h-1 rounded-full {isToday || isSelected ? 'bg-white' : 'bg-[var(--color-today-red)]'}"></span>
                      {/if}
                    </button>
                  {:else}
                    <div></div>
                  {/if}
                {/each}
              </div>
            </div>
          {/if}
        </div>
        
        <!-- Right side: Search toggle and Today button -->
        <div class="flex items-center gap-2 relative z-10">
          {#if isMobileSearchOpen}
            <div class="flex items-center bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg px-2 py-1 max-w-[120px] xs:max-w-[160px] sm:max-w-[200px] relative animate-fadeIn">
              <Search class="w-3.5 h-3.5 text-[var(--color-text-secondary)] shrink-0" />
              <input
                type="text"
                placeholder="Search..."
                bind:value={searchQuery}
                class="bg-transparent text-white text-xs w-full outline-none pl-1.5 pr-5 placeholder:text-[var(--color-text-secondary)]/40 font-mono"
              />
              <button
                onclick={() => { searchQuery = ''; isMobileSearchOpen = false; }}
                class="absolute right-1 text-[var(--color-text-secondary)]/50 hover:text-white p-0.5 cursor-pointer"
              >
                <X class="w-3 h-3" />
              </button>
            </div>
          {:else}
            <button
              onclick={() => isMobileSearchOpen = true}
              class="p-1.5 rounded-lg hover:bg-white/5 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer flex items-center justify-center w-8 h-8 shrink-0"
              title="Search Events"
            >
              <Search class="w-4 h-4" />
            </button>
          {/if}

          <button
            onclick={handleJumpToToday}
            class="flex items-center justify-center hover:bg-white/5 active:bg-white/10 text-white hover:text-white w-8 h-8 rounded-lg font-mono text-sm font-semibold cursor-pointer transition-colors"
            title="Go to Today"
          >
            {new Date().getDate()}
          </button>
        </div>
      </header>
    {:else}
      <!-- Desktop Header -->
      <header class="pl-6 pr-36 py-3 flex items-center justify-between shrink-0 bg-[#0a0a0a] cursor-default select-none relative border-b border-[var(--color-border-hairline)]">
        <!-- Transparent drag handle that stops before WindowControls -->
        <div class="absolute inset-y-0 left-0 right-36" data-tauri-drag-region></div>

        <!-- Left Controls (Dropdown, Today, Arrows, Title) -->
        <div class="flex items-center gap-3 relative z-10">
          
          <!-- View Mode Dropdown Select -->
          <div class="relative z-50">
            <button
              onclick={() => {
                isViewDropdownOpen = !isViewDropdownOpen;
                dropdownSubmenu = 'none';
              }}
              class="bg-[var(--color-canvas-card)] hover:bg-[var(--color-canvas-hover)] border border-[var(--color-border-hairline)] rounded-lg px-3 py-1 text-xs text-white flex items-center gap-1.5 transition-all font-mono font-medium cursor-pointer relative z-10"
            >
              <span class="capitalize">{viewModeLabel}</span>
              <ChevronDown class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
            </button>
            
            {#if isViewDropdownOpen}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="fixed inset-0 z-40" onclick={() => isViewDropdownOpen = false} role="presentation"></div>
              <div class="absolute left-0 mt-1.5 w-52 bg-[#161616] border border-neutral-800 rounded-xl shadow-2xl py-1 z-50 text-xs font-sans">
                {#if dropdownSubmenu === 'none'}
                  <div class="flex flex-col">
                    {#each [
                      { label: 'Day', mode: 'day', shortcut: '1 or D' },
                      { label: 'Week', mode: 'week', shortcut: '0 or W' },
                      { label: 'Month', mode: 'month', shortcut: 'M' },
                    ] as item}
                      <button
                        onclick={() => {
                          viewMode = item.mode as any;
                          isViewDropdownOpen = false;
                        }}
                        class="w-full text-left px-3.5 py-2.5 transition-colors flex items-center justify-between cursor-pointer {viewMode === item.mode ? 'text-white bg-neutral-800/80 font-semibold' : 'text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]'}"
                      >
                        <span>{item.label}</span>
                        <span class="text-[10px] font-mono text-neutral-500">{item.shortcut}</span>
                      </button>
                    {/each}

                    <div class="my-1 border-t border-[var(--color-border-hairline)]"></div>
                    
                    <button
                      onclick={() => dropdownSubmenu = 'number_of_days'}
                      class="w-full text-left px-3.5 py-2.5 text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)] transition-colors flex items-center justify-between cursor-pointer"
                    >
                      <span>Number of days</span>
                      <ChevronRight class="w-3.5 h-3.5 text-neutral-500" />
                    </button>
                    
                    <hr class="border-neutral-800/60 my-1" />
                    
                    <button
                      onclick={() => dropdownSubmenu = 'settings'}
                      class="w-full text-left px-3.5 py-2.5 text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)] transition-colors flex items-center justify-between cursor-pointer"
                    >
                      <span>View settings</span>
                      <ChevronRight class="w-3.5 h-3.5 text-neutral-500" />
                    </button>
                  </div>
                {:else if dropdownSubmenu === 'number_of_days'}
                  <div class="flex flex-col">
                    <button
                      onclick={() => dropdownSubmenu = 'none'}
                      class="w-full text-left px-3 py-2 text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)] transition-colors flex items-center gap-2 cursor-pointer font-medium"
                    >
                      <ChevronLeft class="w-3.5 h-3.5" />
                      <span>Back</span>
                    </button>
                    
                    <div class="my-1 border-t border-[var(--color-border-hairline)]"></div>
                    
                    {#each [
                      { label: '2 days', mode: '2-day', shortcut: '2' },
                      { label: '3 days', mode: '3-day', shortcut: '3' },
                      { label: '4 days', mode: '4-day', shortcut: '4' },
                      { label: '5 days', mode: '5-day', shortcut: '5' },
                      { label: '6 days', mode: '6-day', shortcut: '6' },
                      { label: '7 days', mode: '7-day', shortcut: '7' },
                      { label: '14 days', mode: '14-day', shortcut: '14' },
                    ] as item}
                      <button
                        onclick={() => {
                          viewMode = item.mode as any;
                          isViewDropdownOpen = false;
                          dropdownSubmenu = 'none';
                        }}
                        class="w-full text-left px-3.5 py-2.5 transition-colors flex items-center justify-between cursor-pointer {viewMode === item.mode ? 'text-white bg-neutral-800/80 font-semibold' : 'text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)]'}"
                      >
                        <span>{item.label}</span>
                        <span class="text-[10px] font-mono text-neutral-500">{item.shortcut}</span>
                      </button>
                    {/each}
                  </div>
                {:else if dropdownSubmenu === 'settings'}
                  <div class="flex flex-col">
                    <button
                      onclick={() => dropdownSubmenu = 'none'}
                      class="w-full text-left px-3 py-2 text-[var(--color-text-secondary)] hover:text-white hover:bg-[var(--color-canvas-hover)] transition-colors flex items-center gap-2 cursor-pointer font-medium"
                    >
                      <ChevronLeft class="w-3.5 h-3.5" />
                      <span>Back</span>
                    </button>
                    
                    <div class="my-1 border-t border-[var(--color-border-hairline)]"></div>
                    
                    <button type="button" class="w-full px-3.5 py-2.5 flex items-center justify-between hover:bg-[var(--color-canvas-hover)] cursor-pointer text-left" onclick={() => showWeekends = !showWeekends}>
                      <span class="text-white">Show weekends</span>
                      <div 
                        class="w-7 h-4 rounded-full transition-colors relative cursor-pointer {showWeekends ? 'bg-[#d15b47]' : 'bg-neutral-700'}"
                        aria-hidden="true"
                      >
                        <div class="w-3 h-3 bg-white rounded-full absolute top-0.5 transition-all {showWeekends ? 'right-0.5' : 'left-0.5'}"></div>
                      </div>
                    </button>
                    
                    <button type="button" class="w-full px-3.5 py-2.5 flex items-center justify-between hover:bg-[var(--color-canvas-hover)] cursor-pointer text-left" onclick={() => isDetailsDocked = !isDetailsDocked}>
                      <span class="text-white">Dock Details panel</span>
                      <div 
                        class="w-7 h-4 rounded-full transition-colors relative cursor-pointer {isDetailsDocked ? 'bg-[#d15b47]' : 'bg-neutral-700'}"
                        aria-hidden="true"
                      >
                        <div class="w-3 h-3 bg-white rounded-full absolute top-0.5 transition-all {isDetailsDocked ? 'right-0.5' : 'left-0.5'}"></div>
                      </div>
                    </button>
                    
                    <div class="px-3.5 py-2 space-y-2">
                      <span class="text-white block">Start hour</span>
                      <select
                        bind:value={startHour}
                        class="w-full bg-[#101010] border border-neutral-800 rounded-lg px-2 py-1 text-white outline-none font-mono cursor-pointer"
                      >
                        {#each Array.from({ length: 24 }) as _, i}
                          <option value={i}>{i === 0 ? '12 AM' : i === 12 ? '12 PM' : i > 12 ? `${i - 12} PM` : `${i} AM`}</option>
                        {/each}
                      </select>
                    </div>
                  </div>
                {/if}
              </div>
            {/if}
          </div>

          <!-- Today Navigation Group -->
          <div class="flex items-center bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg overflow-hidden relative z-10">
            <button 
              onclick={() => handleNavigateDate('prev')}
              class="px-2 py-1 hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer border-r border-[var(--color-border-hairline)]"
            >
              <ChevronLeft class="w-3.5 h-3.5" />
            </button>
            <button 
              onclick={handleJumpToToday}
              class="px-3 py-1 hover:bg-[var(--color-canvas-hover)] transition-colors cursor-pointer text-xs font-mono font-medium text-white"
            >
              Today
            </button>
            <button 
              onclick={() => handleNavigateDate('next')}
              class="px-2 py-1 hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer border-l border-[var(--color-border-hairline)]"
            >
              <ChevronRight class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Date Header -->
          <span class="text-sm font-medium text-white font-mono tracking-tight ml-2">
            {headerLabel}
          </span>
        </div>

        <!-- Right side: Search toggle -->
        <div class="flex items-center gap-2 relative z-10">
          {#if isMobileSearchOpen}
            <div class="flex items-center bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg px-2 py-1 max-w-[200px] relative">
              <Search class="w-3.5 h-3.5 text-[var(--color-text-secondary)] shrink-0" />
              <input
                type="text"
                placeholder="Search..."
                bind:value={searchQuery}
                class="bg-transparent text-white text-xs w-full outline-none pl-1.5 pr-5 placeholder:text-neutral-500 font-mono"
              />
              <button
                onclick={() => { searchQuery = ''; isMobileSearchOpen = false; }}
                class="absolute right-1 text-neutral-500 hover:text-white p-0.5 cursor-pointer"
              >
                <X class="w-3 h-3" />
              </button>
            </div>
          {:else}
            <button
              onclick={() => isMobileSearchOpen = true}
              class="p-1.5 rounded-lg hover:bg-white/5 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer flex items-center justify-center w-8 h-8"
              title="Search Events"
            >
              <Search class="w-4 h-4" />
            </button>
          {/if}
        </div>
      </header>
    {/if}

    <!-- Mobile Search Overlay (Task 28) -->
    {#if isMobileSearchOpen && isMobileOrTablet}
      <div class="fixed inset-0 z-[100] bg-[#0a0a0a] flex flex-col p-4 animate-fadeIn">
        <div class="flex items-center gap-3 mb-4">
          <div class="flex-1 flex items-center bg-[#1a1a1a] border border-[var(--color-border-hairline)] rounded-lg px-3 py-2">
            <Search class="w-4 h-4 text-[var(--color-text-secondary)] shrink-0" />
            <!-- svelte-ignore a11y_autofocus -->
            <input
              type="text"
              placeholder="Search events..."
              bind:value={searchQuery}
              class="bg-transparent text-white text-sm w-full outline-none pl-2 font-sans"
              autofocus
            />
          </div>
          <button onclick={() => { searchQuery = ''; isMobileSearchOpen = false; }} class="px-3 py-2 text-[var(--color-text-secondary)] hover:text-white font-medium cursor-pointer">Cancel</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          {#if searchQuery}
            <div class="text-center text-neutral-500 pt-10 text-sm">Searching for "{searchQuery}"...</div>
          {:else}
            <div class="text-center text-neutral-600 pt-10 text-sm">Type to search events</div>
          {/if}
        </div>
      </div>
    {/if}



    <!-- Unified timeline/month/agenda grid view component -->
    <WeekGrid
      events={filteredEvents}
      {selectedDate}
      bind:viewMode
      {showWeekends}
      {startHour}
      selectedEventId={selectedEvent?.id}
      onEventClick={(ev, e) => {
        selectedEvent = ev;
        if (e && viewMode === 'month') {
          clickPosition = { x: e.clientX, y: e.clientY };
        } else {
          clickPosition = null;
          isDetailsDocked = true;
        }
      }}
      onEmptySlotClick={openNewEventPanel}
      onChangeViewMode={(m) => viewMode = m}
      onEventUpdate={(id, updates) => {
        import('@kestrel/shared/api').then(({ updateEvent }) => {
          updateEvent(id, updates).then(() => {
            events = events.map(ev => ev.id === id ? { ...ev, ...updates } : ev);
            showToast('Event updated successfully', 'success');
          }).catch(err => {
            console.error('Failed to update event:', err);
            showToast('Failed to update event', 'error');
          });
        });
      }}
    />
  </div>

  <!-- Event Details Sidebar Peek -->
  {#if selectedEvent}
    <EventPeekPanel
      event={selectedEvent}
      {clickPosition}
      isDocked={isDetailsDocked && !isMobileOrTablet}
      isMobileOrTablet={isMobileOrTablet}
      {accounts}
      onClose={() => selectedEvent = null}
      onSave={(updatedEvent) => {
        handleSaveEvent(updatedEvent);
        selectedEvent = null;
      }}
      onDelete={() => {
        events = events.filter(e => e.id !== selectedEvent.id);
        selectedEvent = null;
      }}
    />
  {:else if isDetailsDocked && !isMobileOrTablet}
    <!-- Dedicated Docked Empty State (Task 34) -->
    <div class="hidden lg:flex flex-col items-center justify-center fixed inset-y-0 right-0 w-80 bg-[#131313] border-l border-[var(--color-border-hairline)] z-40 h-screen text-center p-6 text-[var(--color-text-secondary)] shadow-xl">
      <CalendarIcon class="w-12 h-12 mb-4 opacity-20" />
      <h3 class="font-bold text-white mb-1">No Event Selected</h3>
      <p class="text-xs">Click on any event in the grid to view its details here.</p>
    </div>
  {/if}

  <!-- Creation & Editing Dialog Form Modal Removed in favor of unified EventPeekPanel -->
  
  <!-- Global Toast Notification UI -->
  {#if toastMessage}
    <div class="fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] animate-slide-up flex items-center gap-2 px-4 py-2.5 rounded-xl shadow-2xl {toastMessage.type === 'error' ? 'bg-red-500/90 text-white' : toastMessage.type === 'success' ? 'bg-green-500/90 text-white' : 'bg-[#222222] text-white border border-neutral-700'}">
      <span class="text-sm font-medium">{toastMessage.text}</span>
      <button onclick={() => toastMessage = null} class="ml-2 opacity-70 hover:opacity-100 cursor-pointer p-0.5">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
    </div>
  {/if}
  {/snippet}
</AppShell>

<svelte:window onkeydown={handleGlobalKeydown} />
