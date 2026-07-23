<script lang="ts">
  import { X, Calendar as CalendarIcon, Clock, MapPin, AlignLeft, Users, Bell, Flag, Check, ChevronDown, MoreHorizontal, Square, ArrowRight, Globe, CornerUpLeft, Repeat, User, Video, Link, Trash2 } from 'lucide-svelte';

  export interface EventDetail {
    id?: string;
    title: string;
    date: string;
    startTime: string;
    endTime: string;
    description?: string;
    color: string;
    location?: string;
    category?: string;
    organizer?: string;
    priority?: string;
    status?: string;
    attendees?: { name: string; email: string; rsvp: 'yes' | 'no' | 'maybe' }[];
    rsvpStatus?: 'yes' | 'no' | 'maybe' | 'none';
    isAllDay?: boolean;
    calendarId?: string;
  }

  let {
    event = null,
    clickPosition = null,
    isDocked = false,
    isMobileOrTablet = false,
    accounts = [],
    onClose = () => {},
    onSave = (eventData: any) => {},
    onDelete = () => {}
  } = $props<{
    event?: EventDetail | null;
    clickPosition?: { x: number, y: number } | null;
    isDocked?: boolean;
    isMobileOrTablet?: boolean;
    accounts?: any[];
    onClose?: () => void;
    onSave?: (eventData: any) => void;
    onDelete?: () => void;
  }>();

  let popoverStyle = $derived.by(() => {
    if (isDocked || isMobileOrTablet) return ''; // Docked and Mobile use fixed classes
    
    const PANEL_WIDTH = 340; // Reduced from 450
    const PANEL_HEIGHT = 500; // Estimated height
    const MARGIN = 16;

    if (!clickPosition) return `top: 50%; left: 50%; transform: translate(-50%, -50%); width: ${PANEL_WIDTH}px; max-width: 90vw;`;
    
    // Advanced Bounding Box Math for Floating Panel
    let x = clickPosition.x + 20;
    let y = clickPosition.y - 20;
    let maxH = 500;
    if (typeof window !== 'undefined') {
      if (x + PANEL_WIDTH > window.innerWidth - MARGIN) x = clickPosition.x - PANEL_WIDTH - 20;
      if (x < MARGIN) x = MARGIN;
      
      if (y + PANEL_HEIGHT > window.innerHeight - MARGIN) y = window.innerHeight - PANEL_HEIGHT - MARGIN;
      if (y < MARGIN) y = MARGIN;
      
      maxH = window.innerHeight - y - MARGIN;
    }
    return `top: ${y}px; left: ${x}px; width: ${PANEL_WIDTH}px; max-width: 90vw; max-height: ${maxH}px;`;
  });

  // Form states
  let title = $state('');
  let description = $state('');
  let location = $state('');
  let date = $state('');
  let startTime = $state('10:00');
  let endTime = $state('11:00');
  let priority = $state<'High' | 'Medium' | 'Low' | 'None'>('None');
  let status = $state('Scheduled');
  let category = $state('Work');
  let color = $state('blue');
  let rsvpStatus = $state<'yes' | 'no' | 'maybe' | 'none'>('none');
  let attendeesInput = $state('');
  let isAllDay = $state(false);
  let calendarId = $state('');
  let organizer = $state('');

  let initialSnapshot: any = null;
  let currentEventId: string | undefined = undefined;

  // Hydrate data when component mounts or event changes
  $effect(() => {
    if (event) {
      if (event.id !== currentEventId) {
        initialSnapshot = JSON.parse(JSON.stringify(event));
        currentEventId = event.id;
      }
      
      title = event.title || '';
      description = event.description || '';
      location = event.location || '';
      date = event.date || new Date().toISOString().split('T')[0];
      startTime = event.startTime || '10:00';
      endTime = event.endTime || '11:00';
      priority = (event.priority as any) || 'None';
      status = event.status || 'Scheduled';
      category = event.category || 'Work';
      color = event.color || 'blue';
      rsvpStatus = event.rsvpStatus || 'none';
      isAllDay = event.isAllDay || false;
      calendarId = event.calendarId || (accounts?.length > 0 && accounts[0].calendars?.length > 0 ? accounts[0].calendars[0].id : '');
      organizer = event.organizer || '';
      if (event.attendees && event.attendees.length > 0) {
        attendeesInput = event.attendees.map((a: any) => a.email).join(', ');
      } else {
        attendeesInput = '';
      }
    }
  });

  function handleSave(close = true) {
    if (!title.trim()) return;

    onSave({
      id: event?.id, // Keep the ID so we update the existing event
      title,
      description,
      location,
      date,
      startTime,
      endTime,
      priority,
      status,
      category,
      color,
      isAllDay,
      calendarId,
      organizer,
      rsvpStatus,
      attendees: attendeesInput.split(',').map(email => ({ name: email.trim(), email: email.trim(), rsvp: 'none' })).filter(a => a.email)
    });
    
    if (close) onClose();
  }

  function handleEscape() {
    if (event?.id && initialSnapshot) {
      onSave(initialSnapshot);
    }
    onClose();
  }
</script>

{#if event}
  <!-- Invisible backdrop to catch clicks outside the popover -->
  {#if !isDocked || isMobileOrTablet}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-40 {isMobileOrTablet ? 'bg-black/60 backdrop-blur-sm transition-opacity animate-fadeIn' : 'bg-transparent'}"
      onclick={onClose}
      onkeydown={(e) => { if (e.key === 'Escape') handleEscape(); }}
      tabindex="0"
      role="button"
    ></div>
  {/if}

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="bg-[#1a1a1a] border border-[#2a2a2a] overflow-hidden font-sans shadow-2xl flex flex-col text-sm text-[var(--color-text-primary)] {isMobileOrTablet ? 'fixed bottom-0 left-0 right-0 z-50 rounded-t-2xl max-h-[90vh] overflow-y-auto' : isDocked ? 'fixed inset-y-0 right-0 w-80 border-l z-50 rounded-none h-screen' : 'fixed z-50 max-h-[90vh] rounded-xl animate-fadeIn'}"
    style={popoverStyle}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { 
      if (e.key === 'Escape') handleEscape(); 
      e.stopPropagation(); 
    }}
    role="dialog"
  >
    <!-- Header -->
    <div class="px-3 py-2 flex items-center justify-between">
      <button class="flex items-center gap-1.5 hover:bg-white/5 px-2 py-1.5 rounded cursor-pointer transition-colors">
        <span class="font-bold text-white text-xs">Event</span>
        <ChevronDown class="w-3.5 h-3.5 text-neutral-400" />
      </button>
      <div class="flex items-center gap-1">
        {#if event.id}
          <button onclick={onDelete} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-red-400 transition-colors cursor-pointer">
            <Trash2 class="w-4 h-4" />
          </button>
        {/if}
        <button class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer">
          <MoreHorizontal class="w-4 h-4" />
        </button>
        <button class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer">
          <Square class="w-3 h-3" />
        </button>
        <button onclick={onClose} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Scrollable form area -->
    <div 
      class="p-6 pt-2 space-y-5 flex-1 overflow-y-auto"
      onchange={() => { if (event?.id) handleSave(false); }}
    >
      
      <!-- Title Input Block -->
      <div>
        <input
          id="title"
          type="text"
          placeholder="Add title"
          bind:value={title}
          class="w-full bg-transparent border-none outline-none text-white text-lg placeholder:text-neutral-500 pb-2 border-b border-transparent focus:border-blue-500 transition-colors"
          required
          autocomplete="off"
        />
      </div>

      <!-- Date & Time Block -->
      <div class="space-y-3">
        <div class="flex items-center gap-4">
          <Clock class="w-4 h-4 text-neutral-400 shrink-0" />
          <div class="flex items-center gap-3 flex-1 min-w-0">
            <input type="time" bind:value={startTime} disabled={isAllDay} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white text-xs border-none cursor-pointer w-20" style="color-scheme: dark;" />
            <ArrowRight class="w-3.5 h-3.5 text-neutral-500 shrink-0" />
            <input type="time" bind:value={endTime} disabled={isAllDay} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white text-xs border-none cursor-pointer w-20" style="color-scheme: dark;" />
            {#if !isAllDay}
              <span class="text-neutral-500 text-xs">1h</span>
            {/if}
          </div>
        </div>

        <div class="pl-8">
          <input type="date" bind:value={date} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white text-xs border-none cursor-pointer" style="color-scheme: dark;" />
        </div>

        <div class="pl-8 flex items-center justify-between">
          <label class="flex items-center gap-3 cursor-pointer">
            <!-- Switch toggle -->
            <div class="relative inline-block w-8 h-4">
              <input type="checkbox" bind:checked={isAllDay} class="peer sr-only" />
              <div class="w-full h-full bg-neutral-600 rounded-full peer-checked:bg-blue-400 transition-colors"></div>
              <div class="absolute left-0.5 top-0.5 w-3 h-3 bg-white rounded-full transition-transform peer-checked:translate-x-4"></div>
            </div>
            <span class="text-xs text-white">All-day</span>
          </label>
        </div>

        <div class="pl-8 pt-1 flex items-center justify-between group cursor-pointer">
          <div class="flex items-center gap-3">
            <Globe class="w-4 h-4 text-neutral-500 shrink-0" />
            <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">GMT+5:30 <span class="text-white">Calcutta</span></span>
          </div>
          <CornerUpLeft class="w-3.5 h-3.5 text-neutral-500" />
        </div>

        <div class="pl-8 flex items-center gap-3 group cursor-pointer">
          <Repeat class="w-4 h-4 text-neutral-500 shrink-0" />
          <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">Repeat</span>
        </div>
      </div>

      <!-- Additions Block -->
      <div class="space-y-3">
        <div class="flex items-center gap-4 group cursor-pointer">
          <User class="w-4 h-4 text-neutral-500 shrink-0" />
          <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">Participants and Rooms</span>
        </div>
        
        <div class="flex items-center gap-4 group cursor-pointer">
          <Video class="w-4 h-4 text-neutral-500 shrink-0" />
          <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">Conferencing</span>
        </div>

        <div class="flex items-center gap-4 group">
          <MapPin class="w-4 h-4 text-neutral-500 shrink-0" />
          <input type="text" placeholder="Location" bind:value={location} class="w-full bg-transparent border-none outline-none text-xs text-white placeholder:text-neutral-400" />
        </div>
      </div>

      <!-- Links and Description Block -->
      <div class="space-y-3">
        <div class="flex items-center gap-4 group cursor-pointer">
          <Link class="w-4 h-4 text-neutral-500 shrink-0" />
          <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">Add links and attachments</span>
        </div>
        <div class="pl-8 pt-1">
          <textarea
            placeholder="Description"
            bind:value={description}
            rows="3"
            class="w-full bg-transparent border-none outline-none text-xs text-white placeholder:text-neutral-500 resize-none"
          ></textarea>
        </div>
      </div>

      <!-- Calendar & Visibility Block -->
      <div class="space-y-4">
        <div class="flex items-center gap-4">
          <div class="w-3.5 h-3.5 rounded shrink-0" style="background-color: {color};"></div>
          <div class="flex-1 min-w-0 flex items-center gap-3">
            <select bind:value={calendarId} class="w-full bg-transparent border-none outline-none text-xs text-white cursor-pointer hover:bg-white/5 p-1 rounded truncate">
              {#each (accounts || []) as acc}
                <optgroup label={acc.email} class="bg-[#131313] text-neutral-400">
                  {#each acc.calendars as cal}
                    <option value={cal.id} class="text-white bg-[#1a1a1a]">{cal.name}</option>
                  {/each}
                </optgroup>
              {/each}
            </select>
          </div>
        </div>
        
        <div class="pl-8 flex items-center gap-6">
          <span class="text-xs text-white">Busy</span>
          <span class="text-xs text-white">Default visibility</span>
        </div>

        <div class="flex items-center gap-4 group cursor-pointer pt-1">
          <Bell class="w-4 h-4 text-neutral-500 shrink-0" />
          <span class="text-xs text-neutral-400 group-hover:text-white transition-colors">Reminders</span>
        </div>
      </div>

    </div>

    <!-- Footer Actions -->
    {#if !event.id}
      <div class="px-6 py-4 flex items-center justify-end">
        <button 
          onclick={() => handleSave(true)} 
          class="px-5 py-2 rounded-full bg-blue-500 hover:bg-blue-600 text-white text-xs font-bold transition-colors cursor-pointer shadow-md"
        >
          Save
        </button>
      </div>
    {/if}
  </div>
{/if}
