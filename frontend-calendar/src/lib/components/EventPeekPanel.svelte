<script lang="ts">
  import { X, Calendar as CalendarIcon, Edit2, Sparkles, ChevronRight, ExternalLink, Clock, MapPin, AlignLeft, Users, Bell, Flag, Check, X as XIcon, HelpCircle, ChevronDown, MoreHorizontal, Square, ArrowRight, Globe, CornerUpLeft, Repeat, User, Video, Link, Trash2 } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

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
    attendees?: { name?: string; email: string; status?: 'accepted' | 'declined' | 'tentative' | 'needsAction' }[];
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
  import { ContactAutocomplete } from '@kestrel/shared';
  type SelectedContact = {
    name?: string | null;
    email: string;
    avatar_url?: string | null;
    status?: string;
    rsvp?: string;
  };
  let attendeeEmails = $state<string[]>([]);
  let attendees = $state<SelectedContact[]>([]);
  let isAllDay = $state(false);
  let calendarId = $state('');
  let organizer = $state('');
  let isEditing = $state(false);
  let initialSnapshot: any = null;
  let currentEventId: string | undefined = undefined;

  // Hydrate data when component mounts or event changes
  $effect(() => {
    if (event) {
      if (event.id !== currentEventId) {
        isEditing = !event.id;
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
        attendeeEmails = event.attendees.map((a: any) => a.email);
        attendees = [...event.attendees];
      } else {
        attendeeEmails = [];
        attendees = [];
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
      attendees
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
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-40 {isMobileOrTablet ? 'bg-black/60 backdrop-blur-sm' : 'bg-transparent'}"
      onclick={onClose}
      onkeydown={(e) => { if (e.key === 'Escape') handleEscape(); }}
      tabindex="0"
      role="button"
    ></div>
  {/if}

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    transition:fly={{ x: isDocked && !isMobileOrTablet ? 50 : 0, y: isMobileOrTablet || (!isDocked && !isMobileOrTablet) ? 30 : 0, duration: 300, easing: cubicOut }}
    class="bg-[#1a1a1a] border border-[#2a2a2a] overflow-hidden font-sans shadow-2xl flex flex-col text-sm text-[var(--color-text-primary)] {isMobileOrTablet ? 'fixed bottom-0 left-0 right-0 z-50 rounded-t-2xl max-h-[90vh] overflow-y-auto' : isDocked ? 'fixed inset-y-0 right-0 w-80 border-l z-50 rounded-none h-screen' : 'fixed z-50 max-h-[90vh] rounded-xl'}"
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

        <button onclick={onClose} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>


    <!-- Scrollable content area -->
    <div
      class="p-6 pt-2 space-y-5 flex-1 overflow-y-auto"
      onchange={() => { if (event?.id && isEditing) handleSave(false); }}
    >
      {#if isEditing}
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
          <div class="flex items-center gap-4 group">
            <User class="w-4 h-4 text-neutral-500 shrink-0" />
            <div class="flex-1 w-full"><ContactAutocomplete bind:recipients={attendeeEmails} bind:contacts={attendees} placeholder="Add guests..." /></div>
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

          <div class="flex items-center gap-4 group pt-1">
            <Bell class="w-4 h-4 text-neutral-500 shrink-0" />
            <select class="w-full bg-transparent border-none outline-none text-xs text-white cursor-pointer hover:bg-white/5 p-1 rounded">
              <option value="10m">10 minutes before</option>
              <option value="30m">30 minutes before</option>
              <option value="1h">1 hour before</option>
              <option value="none">No reminder</option>
            </select>
          </div>
        </div>
      {:else}
        <!-- VIEW MODE DETAILS VIEW -->
        <div class="space-y-5">

          <!-- Event Title Block -->
          <div class="space-y-1">
            <h3 class="text-sm font-bold text-white leading-snug">
              {event.title}
            </h3>
          </div>

          <!-- Planned Execution Date Block - BORDERLESS -->
          <div class="space-y-2.5 bg-neutral-900/30 p-3.5 rounded-xl">

            <div class="flex items-start gap-2.5">
              <Clock class="w-3.5 h-3.5 text-neutral-400 mt-0.5 flex-shrink-0" />
              <div class="space-y-0.5">
                <div class="text-xs text-white font-bold flex items-center gap-2">
                  {event.isAllDay ? "All Day" : `${event.startTime} → ${event.endTime}`}
                  {#if !event.isAllDay}
                     <span class="text-[10px] text-neutral-500 font-normal font-mono">(1h)</span>
                  {/if}
                </div>
                <div class="text-[11px] text-neutral-400 font-mono">
                  {new Date(event.date).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' })}
                </div>
              </div>
            </div>

            <!-- Propose New Time Button -->
            <button
              type="button"
              class="w-full mt-2 py-1.5 px-2.5 rounded-lg bg-[#1a1a1a]/60 hover:bg-neutral-800/80 text-[10px] font-semibold text-neutral-200 transition-colors cursor-pointer flex items-center justify-between"
            >
              <span>Propose new time</span>
              <ExternalLink class="w-3 h-3 text-neutral-400" />
            </button>

            <!-- Timezone details -->
            <div class="flex items-center justify-between mt-2 pt-2 border-t border-neutral-800/20 text-[9px] text-neutral-500 font-mono">
              <div class="flex items-center gap-1.5">
                <Globe class="w-3.5 h-3.5 text-neutral-500" />
                <span>Calcutta Time (GMT+5:30)</span>
              </div>
              <CornerUpLeft class="w-3 h-3 text-neutral-500" />
            </div>

          </div>

          <!-- Organizer & Attendees block -->
          {#if event.organizer || (event.attendees && event.attendees.length > 0)}
            <div class="space-y-2.5 pt-1">

              <!-- Organizer -->
              {#if event.organizer}
                <div class="space-y-1">
                  <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Organizer</span>
                  <div class="flex items-center gap-2 bg-neutral-900/30 p-2.5 rounded-xl">
                    <div class="w-7 h-7 rounded-full bg-neutral-800 text-neutral-300 flex items-center justify-center text-xs font-bold font-mono">
                      {event.organizer.charAt(0).toUpperCase()}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="text-xs text-white font-semibold truncate">{event.organizer}</div>
                      <div class="text-[10px] text-neutral-500 font-mono">Organizer</div>
                    </div>
                  </div>
                </div>
              {/if}

              <!-- Attendees List -->
              {#if event.attendees && event.attendees.length > 0}
                <div class="space-y-1">
                  <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Attendees</span>
                  <div class="space-y-1.5">
                    {#each event.attendees as att}
                      <div class="flex items-center gap-2 bg-neutral-900/30 p-2.5 rounded-xl">
                        <div class="relative">
                          <div class="w-7 h-7 rounded-full bg-emerald-800/20 border border-emerald-500/10 text-emerald-400 flex items-center justify-center text-xs font-bold font-mono">
                            {(att.name || att.email).charAt(0).toUpperCase()}
                          </div>
                          {#if att.status === 'accepted'}
                            <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-emerald-500 rounded-full border border-[#131313] flex items-center justify-center">
                              <Check class="w-2 h-2 text-white stroke-[3.5]" />
                            </div>
                          {:else if att.status === 'declined'}
                            <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-rose-500 rounded-full border border-[#131313] flex items-center justify-center">
                              <XIcon class="w-2 h-2 text-white stroke-[3.5]" />
                            </div>
                          {:else if att.status === 'tentative'}
                            <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-amber-500 rounded-full border border-[#131313] flex items-center justify-center">
                              <HelpCircle class="w-2 h-2 text-white stroke-[3.5]" />
                            </div>
                          {/if}
                        </div>
                        <div class="flex-1 min-w-0">
                          <div class="text-xs text-white font-semibold truncate">{att.name || att.email.split('@')[0]}</div>
                          <div class="text-[10px] text-neutral-500 font-mono truncate">{att.email}</div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Interactive RSVP Picker Pill Container -->
              {#if event.rsvpStatus}
                <div class="space-y-1">
                  <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Your RSVP Status</span>
                  <div class="flex items-center gap-1.5 bg-[#1a1a1a]/60 p-1.5 rounded-xl">
                    <button class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {event.rsvpStatus === 'yes' ? 'bg-emerald-500/15 text-emerald-400 font-bold' : 'text-neutral-500 hover:text-white'}">Yes</button>
                    <button class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {event.rsvpStatus === 'no' ? 'bg-rose-500/15 text-rose-400 font-bold' : 'text-neutral-500 hover:text-white'}">No</button>
                    <button class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {event.rsvpStatus === 'maybe' ? 'bg-amber-500/15 text-amber-400 font-bold' : 'text-neutral-500 hover:text-white'}">Maybe</button>
                    <div class="w-px h-5 bg-neutral-800 mx-1"></div>
                    <button onclick={() => isEditing = true} class="p-1.5 rounded-lg text-neutral-500 hover:text-white hover:bg-neutral-800 transition-colors cursor-pointer" title="Edit Response">
                      <Edit2 class="w-3 h-3" />
                    </button>
                  </div>
                </div>
              {/if}

            </div>
          {/if}

          <!-- AI notes and online meeting shortcuts -->
          <div class="space-y-2 pt-3 border-t border-neutral-800/20">

            <button
              type="button"
              class="w-full py-2 px-3 rounded-xl bg-gradient-to-r from-purple-500/5 to-indigo-500/5 hover:from-purple-500/10 hover:to-indigo-500/10 text-xs font-semibold text-purple-300 transition-all cursor-pointer flex items-center justify-between"
            >
              <div class="flex items-center gap-2">
                <Sparkles class="w-3.5 h-3.5 text-purple-400 animate-pulse" />
                <span>Add AI meeting notes</span>
              </div>
              <ChevronRight class="w-3.5 h-3.5 text-purple-400/70" />
            </button>

          </div>

          <!-- Other attributes (Priority, Location, etc.) -->
          <div class="space-y-3 pt-3 border-t border-neutral-800/20">

            <!-- Priority Field -->
            <div class="flex items-center justify-between text-xs py-1.5">
              <div class="flex items-center gap-1 text-neutral-500 font-mono text-[10px]">
                <Flag class="w-3.5 h-3.5 text-neutral-500/60" />
                <span>Priority</span>
              </div>
              {#if event.priority && event.priority !== 'None'}
                <span class="px-2 py-0.5 rounded-full text-[9px] font-mono font-semibold {event.priority === 'High' ? 'bg-rose-500/15 text-rose-400' : event.priority === 'Medium' ? 'bg-amber-500/15 text-amber-400' : 'bg-blue-500/15 text-blue-400'}">
                  {event.priority}
                </span>
              {:else}
                <span class="text-neutral-600 font-mono text-[10px]">None</span>
              {/if}
            </div>

            <!-- Location Field -->
            {#if event.location}
              <div class="space-y-1">
                <div class="flex items-center gap-1 text-neutral-500 font-mono text-[10px]">
                  <MapPin class="w-3.5 h-3.5 text-neutral-500/60" />
                  <span>Location</span>
                </div>
                <div class="text-xs text-white bg-neutral-900/30 rounded-lg p-2.5 font-medium">
                  {event.location}
                </div>
              </div>
            {/if}

          </div>

          <!-- Description / Notes text -->
          {#if event.description}
            <div class="space-y-1.5 pt-3 border-t border-neutral-800/20">
              <div class="flex items-center gap-1 text-neutral-500 font-mono text-[10px]">
                <AlignLeft class="w-3.5 h-3.5 text-neutral-500/60" />
                <span>Notes & Description</span>
              </div>
              <p class="text-xs text-neutral-300 leading-relaxed bg-neutral-900/20 rounded-xl p-3.5 max-h-40 overflow-y-auto whitespace-pre-line">
                {event.description}
              </p>
            </div>
          {/if}

          <!-- CTA Action button (Manage in Workspace) -->
          <div class="pt-2">
            <button
              type="button"
              class="w-full py-2 rounded-lg bg-neutral-900/80 hover:bg-neutral-800/80 text-xs font-semibold text-white transition-colors cursor-pointer flex items-center justify-center gap-2"
            >
              <ExternalLink class="w-3.5 h-3.5" />
              <span>Manage in Workspace</span>
            </button>
          </div>

        </div>
      {/if}
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
