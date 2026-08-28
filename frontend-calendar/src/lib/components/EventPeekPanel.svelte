<script lang="ts">
  import { X, Calendar as CalendarIcon, Edit2, ChevronRight, ExternalLink, Clock, MapPin, AlignLeft, Users, Bell, Flag, Check, X as XIcon, HelpCircle, ChevronDown, MoreHorizontal, Square, ArrowRight, Globe, CornerUpLeft, Repeat, User, Video, Link, Trash2 } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { detectConferenceLink } from '@kestrel/shared';
  import { openUrl } from '@tauri-apps/plugin-opener';

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
    attendees?: { name?: string; email: string; status?: 'accepted' | 'declined' | 'tentative' | 'needsAction'; rsvp?: 'yes' | 'no' | 'maybe' }[];
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

    const PANEL_WIDTH = 400; // Comfortable width for desktop
    const PANEL_HEIGHT = 560; // Estimated height
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

  function updateRsvp(newStatus: string) {
    if (!event || !event.id) return;
    const newRsvpStatus = newStatus === 'accepted' ? 'yes' : newStatus === 'declined' ? 'no' : 'maybe';
    event.rsvpStatus = newRsvpStatus;
    import('@kestrel/shared/api').then(({ updateEvent }) => {
      (updateEvent as any)(event!.id!, { status: newStatus }).catch(console.error);
    });
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
    <div class="px-4 py-3 flex items-center justify-between border-b border-neutral-800/40">
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded-full shrink-0" style="background-color: {color || '#3b82f6'};"></div>
        <span class="font-bold text-white text-xs">{category || 'Event'}</span>
      </div>
      <div class="flex items-center gap-1">
        {#if event.id}
          <button onclick={onDelete} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-red-400 transition-colors cursor-pointer" title="Delete event">
            <Trash2 class="w-4 h-4" />
          </button>
        {/if}

        <button onclick={onClose} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer" title="Close">
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Scrollable Organized Center Peek Body -->
    <div class="px-5 py-4 space-y-4 flex-1 overflow-y-auto">
      <!-- Title (Inline Editable) -->
      <div>
        <input
          type="text"
          bind:value={title}
          placeholder="Untitled Event"
          oninput={() => { if (event?.id) handleSave(false); }}
          class="w-full bg-transparent border-none outline-none text-base font-bold text-white leading-snug hover:bg-white/5 p-1 -m-1 rounded transition-colors placeholder:text-neutral-500"
        />
      </div>

      <!-- Planned Execution Date / Time Block (Inline Editable) -->
      <div class="space-y-2.5 bg-neutral-900/50 p-3 rounded-xl border border-neutral-800/40">
        <div class="flex items-start gap-2.5">
          <Clock class="w-4 h-4 text-neutral-400 mt-1 shrink-0" />
          <div class="space-y-1.5 flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <input
                type="time"
                bind:value={startTime}
                disabled={isAllDay}
                onchange={() => { if (event?.id) handleSave(false); }}
                class="bg-transparent hover:bg-white/5 px-1.5 py-1 rounded outline-none text-white text-sm font-bold border-none cursor-pointer"
                style="color-scheme: dark; min-width: 95px;"
              />
              <span class="text-neutral-500 text-xs">→</span>
              <input
                type="time"
                bind:value={endTime}
                disabled={isAllDay}
                onchange={() => { if (event?.id) handleSave(false); }}
                class="bg-transparent hover:bg-white/5 px-1.5 py-1 rounded outline-none text-white text-sm font-bold border-none cursor-pointer"
                style="color-scheme: dark; min-width: 95px;"
              />
              {#if !isAllDay}
                <span class="text-[11px] text-neutral-500 font-normal font-mono">(1h)</span>
              {/if}
            </div>

            <div class="flex items-center justify-between gap-2 pt-0.5">
              <input
                type="date"
                bind:value={date}
                onchange={() => { if (event?.id) handleSave(false); }}
                class="bg-transparent hover:bg-white/5 px-1.5 py-1 rounded outline-none text-neutral-300 text-xs font-mono border-none cursor-pointer"
                style="color-scheme: dark;"
              />
              <label class="flex items-center gap-2 cursor-pointer pr-1">
                <div class="relative inline-block w-7 h-3.5">
                  <input
                    type="checkbox"
                    bind:checked={isAllDay}
                    onchange={() => { if (event?.id) handleSave(false); }}
                    class="peer sr-only"
                  />
                  <div class="w-full h-full bg-neutral-600 rounded-full peer-checked:bg-blue-400 transition-colors"></div>
                  <div class="absolute left-0.5 top-0.5 w-2.5 h-2.5 bg-white rounded-full transition-transform peer-checked:translate-x-3.5"></div>
                </div>
                <span class="text-[10px] text-neutral-400 font-mono">All-day</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Timezone details -->
        <div class="flex items-center justify-between pt-2 border-t border-neutral-800/30 text-[10px] text-neutral-500 font-mono">
          <div class="flex items-center gap-1.5">
            <Globe class="w-3.5 h-3.5 text-neutral-500" />
            <span>Calcutta Time (GMT+5:30)</span>
          </div>
          <CornerUpLeft class="w-3.5 h-3.5 text-neutral-500" />
        </div>
      </div>

      <!-- Organizer & Attendees Block -->
      <div class="space-y-2.5 pt-0.5">
        {#if organizer}
          <div class="space-y-1">
            <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Organizer</span>
            <div class="flex items-center gap-2.5 bg-neutral-900/40 p-2.5 rounded-xl border border-neutral-800/30">
              <div class="w-7 h-7 rounded-full bg-neutral-800 text-neutral-300 flex items-center justify-center text-xs font-bold font-mono">
                {organizer.charAt(0).toUpperCase()}
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-xs text-white font-semibold truncate">{organizer}</div>
                <div class="text-[10px] text-neutral-500 font-mono">Organizer</div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Attendees Profile List -->
        <div class="space-y-1.5">
          <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Attendees</span>
          
          {#if attendees && attendees.length > 0}
            <div class="space-y-1.5">
              {#each attendees as att}
                <div class="flex items-center gap-2.5 bg-neutral-900/40 p-2.5 rounded-xl border border-neutral-800/30 group">
                  <div class="relative">
                    <div class="w-7 h-7 rounded-full bg-emerald-800/20 border border-emerald-500/10 text-emerald-400 flex items-center justify-center text-xs font-bold font-mono">
                      {(att.name || att.email).charAt(0).toUpperCase()}
                    </div>
                    {#if att.status === 'accepted' || att.rsvp === 'yes'}
                      <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-emerald-500 rounded-full border border-[#131313] flex items-center justify-center">
                        <Check class="w-2 h-2 text-white stroke-[3.5]" />
                      </div>
                    {:else if att.status === 'declined' || att.rsvp === 'no'}
                      <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-rose-500 rounded-full border border-[#131313] flex items-center justify-center">
                        <XIcon class="w-2 h-2 text-white stroke-[3.5]" />
                      </div>
                    {:else if att.status === 'tentative' || att.rsvp === 'maybe'}
                      <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-amber-500 rounded-full border border-[#131313] flex items-center justify-center">
                        <HelpCircle class="w-2 h-2 text-white stroke-[3.5]" />
                      </div>
                    {/if}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="text-xs text-white font-semibold truncate">{att.name || att.email.split('@')[0]}</div>
                    <div class="text-[10px] text-neutral-500 font-mono truncate">{att.email}</div>
                  </div>
                  <button
                    onclick={() => {
                      attendees = attendees.filter(a => a.email !== att.email);
                      attendeeEmails = attendeeEmails.filter(e => e !== att.email);
                      if (event?.id) handleSave(false);
                    }}
                    class="p-1 text-neutral-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all rounded hover:bg-neutral-800 cursor-pointer"
                    title="Remove attendee"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Quick Add Attendee Autocomplete -->
          <div class="pt-1">
            <ContactAutocomplete
              bind:recipients={attendeeEmails}
              bind:contacts={attendees}
              showChips={false}
              placeholder="+ Add attendee by name or email..."
            />
          </div>
        </div>

        <!-- RSVP Response Pill -->
        <div class="space-y-1">
          <span class="text-[10px] font-mono uppercase tracking-wider text-neutral-500">Your RSVP Status</span>
          <div class="flex items-center gap-1 bg-[#1a1a1a]/80 p-1.5 rounded-xl border border-neutral-800/40">
            <button onclick={() => updateRsvp('accepted')} class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {rsvpStatus === 'yes' ? 'bg-emerald-500/15 text-emerald-400 font-bold' : 'text-neutral-500 hover:text-white'}">Yes</button>
            <button onclick={() => updateRsvp('declined')} class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {rsvpStatus === 'no' ? 'bg-rose-500/15 text-rose-400 font-bold' : 'text-neutral-500 hover:text-white'}">No</button>
            <button onclick={() => updateRsvp('tentative')} class="flex-1 py-1 text-[11px] font-bold rounded-lg transition-colors cursor-pointer {rsvpStatus === 'maybe' ? 'bg-amber-500/15 text-amber-400 font-bold' : 'text-neutral-500 hover:text-white'}">Maybe</button>
          </div>
        </div>
      </div>

      <!-- Location / Conference Field (Inline Editable) -->
      <div class="space-y-1 pt-2 border-t border-neutral-800/20">
        <div class="flex items-center justify-between text-[10px] font-mono text-neutral-500 uppercase tracking-wider">
          <div class="flex items-center gap-1">
            <MapPin class="w-3.5 h-3.5 text-neutral-500/60" />
            <span>Location</span>
          </div>
          {#if location}
            {@const confLink = detectConferenceLink(location)}
            {#if confLink}
              <button
                onclick={() => openUrl(confLink.url)}
                class="px-2.5 py-0.5 rounded bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 text-[10px] font-bold transition-colors cursor-pointer"
              >
                Join {confLink.provider}
              </button>
            {/if}
          {/if}
        </div>
        <input
          type="text"
          placeholder="Add location or video link..."
          bind:value={location}
          oninput={() => { if (event?.id) handleSave(false); }}
          class="w-full text-xs text-white bg-neutral-900/40 rounded-lg p-2.5 font-medium border border-neutral-800/30 outline-none hover:border-neutral-700 transition-colors placeholder:text-neutral-500"
        />
      </div>

      <!-- Notes / Description (Inline Editable) -->
      <div class="space-y-1 pt-2 border-t border-neutral-800/20">
        <div class="flex items-center gap-1 text-neutral-500 font-mono text-[10px]">
          <AlignLeft class="w-3.5 h-3.5 text-neutral-500/60" />
          <span>Notes & Description</span>
        </div>
        <textarea
          placeholder="Add notes or description..."
          bind:value={description}
          oninput={() => { if (event?.id) handleSave(false); }}
          rows="3"
          class="w-full text-xs text-neutral-300 leading-relaxed bg-neutral-900/30 rounded-xl p-3 max-h-36 overflow-y-auto whitespace-pre-line border border-neutral-800/30 outline-none hover:border-neutral-700 transition-colors placeholder:text-neutral-500 resize-none"
        ></textarea>
      </div>

      <!-- Priority & Calendar Attributes -->
      <div class="space-y-2 pt-2 border-t border-neutral-800/20">
        <div class="flex items-center justify-between text-xs py-1">
          <div class="flex items-center gap-1 text-neutral-500 font-mono text-[10px]">
            <Flag class="w-3.5 h-3.5 text-neutral-500/60" />
            <span>Priority</span>
          </div>
          <select
            bind:value={priority}
            onchange={() => { if (event?.id) handleSave(false); }}
            class="bg-neutral-800 hover:bg-neutral-700 text-white text-[10px] font-mono font-semibold rounded px-2 py-0.5 outline-none border border-neutral-700 cursor-pointer"
          >
            <option value="None">None</option>
            <option value="Low">Low</option>
            <option value="Medium">Medium</option>
            <option value="High">High</option>
          </select>
        </div>

        <div class="flex items-center justify-between text-xs py-1">
          <span class="text-neutral-500 font-mono text-[10px]">Calendar</span>
          <select
            bind:value={calendarId}
            onchange={() => { if (event?.id) handleSave(false); }}
            class="bg-neutral-800 hover:bg-neutral-700 text-white text-xs rounded px-2.5 py-1 outline-none border border-neutral-700 cursor-pointer"
          >
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
    </div>

    <!-- Footer Actions (Save button only shown when creating a new event) -->
    {#if !event.id}
      <div class="px-5 py-3 flex items-center justify-end border-t border-neutral-800/40">
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
