<script lang="ts">
  import { X, Calendar as CalendarIcon, Clock, MapPin, AlignLeft, Users, Bell, Flag, Check, ChevronDown, MoreHorizontal, Square, ArrowRight, Globe, CornerUpLeft, Repeat, User, Video, Link } from 'lucide-svelte';
  import { ContactAutocomplete } from '@kestrel/shared';

  export interface CalendarEvent {
    id?: string;
    title: string;
    date: string;
    startTime: string;
    endTime: string;
    description?: string;
    color: string;
    location?: string;
    category?: string;
    calendarId: string;
    status?: string;
    priority?: string;
    isAllDay?: boolean;
    rsvpStatus?: 'yes' | 'no' | 'maybe' | 'none';
  }

  let {
    isOpen = false,
    onClose = () => {},
    onSave = (eventData: any) => {},
    selectedDateStr = '',
    accounts = []
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSave?: (eventData: any) => void;
    selectedDateStr?: string;
    accounts?: any[];
  }>();

  // Form states matching prototype
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

  // Hydrate date when modal opens
  $effect(() => {
    if (isOpen) {
      date = selectedDateStr || new Date().toISOString().split('T')[0];
      if (!calendarId && accounts?.length > 0 && accounts[0].calendars?.length > 0) {
        calendarId = accounts[0].calendars[0].id;
      }
    }
  });

  const COLOR_OPTIONS = [
    { name: 'blue', hex: '#2383e2', dot: 'bg-blue-500' },
    { name: 'purple', hex: '#8a4bf5', dot: 'bg-purple-500' },
    { name: 'green', hex: '#0fa35c', dot: 'bg-emerald-500' },
    { name: 'orange', hex: '#df6a14', dot: 'bg-orange-500' },
    { name: 'rose', hex: '#e03e3e', dot: 'bg-rose-500' },
    { name: 'amber', hex: '#dfab00', dot: 'bg-amber-500' },
    { name: 'teal', hex: '#0fa3b1', dot: 'bg-teal-500' }
  ];

  function handleSave() {
    if (!title.trim()) return;

    onSave({
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

    // Reset
    title = '';
    description = '';
    location = '';
    priority = 'None';
    status = 'Scheduled';
    category = 'Work';
    rsvpStatus = 'none';
    attendees = [];
    attendeeEmails = [];
    isAllDay = false;
    organizer = '';
    onClose();
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs font-sans"
    onclick={onClose}
  >
    <div 
      class="w-full max-w-lg bg-[#1a1a1a] border border-[#2a2a2a] rounded-xl shadow-2xl flex flex-col overflow-hidden text-sm text-[var(--color-text-primary)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-3 py-2 flex items-center justify-between">
        <button class="flex items-center gap-1.5 hover:bg-white/5 px-2 py-1.5 rounded cursor-pointer transition-colors">
          <span class="font-bold text-white text-xs">Event</span>
          <ChevronDown class="w-3.5 h-3.5 text-neutral-400" />
        </button>
        <div class="flex items-center gap-1">

          <button onclick={onClose} class="p-1.5 rounded hover:bg-white/5 text-neutral-400 hover:text-white transition-colors cursor-pointer">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Scrollable form area -->
      <div class="p-6 pt-2 space-y-5 max-h-[80vh] overflow-y-auto">
        
        <!-- Title Input Block -->
        <div>
          <input
            id="title"
            type="text"
            placeholder="Add title"
            bind:value={title}
            class="w-full bg-transparent border-none outline-none text-white text-xl font-medium placeholder:text-neutral-500 pb-2 border-b border-transparent focus:border-blue-500 transition-colors"
            required
            autocomplete="off"
          />
        </div>

        <!-- Date & Time Block -->
        <div class="space-y-3 border-b border-neutral-800 pb-4">
          <div class="flex items-center gap-4">
            <Clock class="w-4 h-4 text-neutral-400 shrink-0" />
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <input type="time" bind:value={startTime} disabled={isAllDay} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white font-medium text-xs border-none cursor-pointer w-20" style="color-scheme: dark;" />
              <ArrowRight class="w-3.5 h-3.5 text-neutral-500 shrink-0" />
              <input type="time" bind:value={endTime} disabled={isAllDay} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white font-medium text-xs border-none cursor-pointer w-20" style="color-scheme: dark;" />
              {#if !isAllDay}
                <span class="text-neutral-500 text-xs font-medium">1h</span>
              {/if}
            </div>
          </div>

          <div class="pl-8">
            <input type="date" bind:value={date} class="bg-transparent hover:bg-white/5 p-1 rounded outline-none text-white font-medium text-xs border-none cursor-pointer" style="color-scheme: dark;" />
          </div>

          <div class="pl-8 flex items-center justify-between">
            <label class="flex items-center gap-3 cursor-pointer">
              <!-- Switch toggle -->
              <div class="relative inline-block w-8 h-4">
                <input type="checkbox" bind:checked={isAllDay} class="peer sr-only" />
                <div class="w-full h-full bg-neutral-600 rounded-full peer-checked:bg-blue-400 transition-colors"></div>
                <div class="absolute left-0.5 top-0.5 w-3 h-3 bg-white rounded-full transition-transform peer-checked:translate-x-4"></div>
              </div>
              <span class="text-sm font-semibold text-white">All-day</span>
            </label>
          </div>

          <div class="pl-8 pt-1 flex items-center justify-between group cursor-pointer">
            <div class="flex items-center gap-3">
              <Globe class="w-4 h-4 text-neutral-500 shrink-0" />
              <span class="text-xs font-semibold text-neutral-400 group-hover:text-white transition-colors">GMT+5:30 <span class="text-white">Calcutta</span></span>
            </div>
            <CornerUpLeft class="w-3.5 h-3.5 text-neutral-500" />
          </div>

          <div class="pl-8 flex items-center gap-3 group cursor-pointer">
            <Repeat class="w-4 h-4 text-neutral-500 shrink-0" />
            <span class="text-xs font-semibold text-neutral-400 group-hover:text-white transition-colors">Repeat</span>
          </div>
        </div>

        <!-- Additions Block -->
        <div class="space-y-3 border-b border-neutral-800 pb-4">
          <div class="flex items-start gap-4 group">
            <User class="w-4 h-4 text-neutral-500 shrink-0 mt-2" />
            <div class="flex-1 w-full max-w-[calc(100%-2rem)]">
              <ContactAutocomplete bind:recipients={attendeeEmails} bind:contacts={attendees} placeholder="Add participants..." />
            </div>
          </div>
          
          <div class="flex items-center gap-4 group cursor-pointer">
            <Video class="w-4 h-4 text-neutral-500 shrink-0" />
            <span class="text-sm font-semibold text-neutral-400 group-hover:text-white transition-colors">Conferencing</span>
          </div>

          <div class="flex items-center gap-4 group">
            <MapPin class="w-4 h-4 text-neutral-500 shrink-0" />
            <input type="text" placeholder="Location" bind:value={location} class="w-full bg-transparent border-none outline-none text-sm font-semibold text-white placeholder:text-neutral-400" />
          </div>
        </div>

        <!-- Links and Description Block -->
        <div class="space-y-3 border-b border-neutral-800 pb-4">
          <div class="flex items-center gap-4 group cursor-pointer">
            <Link class="w-4 h-4 text-neutral-500 shrink-0" />
            <span class="text-sm font-semibold text-neutral-400 group-hover:text-white transition-colors">Add links and attachments</span>
          </div>
          <div class="pl-8 pt-1">
            <textarea
              placeholder="Description"
              bind:value={description}
              rows="3"
              class="w-full bg-transparent border-none outline-none text-sm font-semibold text-white placeholder:text-neutral-500 resize-none"
            ></textarea>
          </div>
        </div>

        <!-- Calendar & Visibility Block -->
        <div class="space-y-4 pb-4">
          <div class="flex items-center gap-4">
            <div class="w-3.5 h-3.5 rounded bg-blue-400 border border-white/10 shrink-0"></div>
            <div class="flex-1 min-w-0 flex items-center gap-3">
              <select bind:value={calendarId} class="w-full bg-transparent border-none outline-none text-sm font-semibold text-white cursor-pointer hover:bg-white/5 p-1 rounded truncate">
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
            <span class="text-xs font-semibold text-white">Busy</span>
            <span class="text-xs font-semibold text-white">Default visibility</span>
          </div>

          <div class="flex items-center gap-4 group cursor-pointer pt-1">
            <Bell class="w-4 h-4 text-neutral-500 shrink-0" />
            <span class="text-sm font-semibold text-neutral-400 group-hover:text-white transition-colors">Reminders</span>
          </div>
        </div>

      </div>

      <!-- Footer Actions -->
      <div class="px-6 py-4 flex items-center justify-end">
        <button 
          onclick={handleSave} 
          class="px-5 py-2 rounded-full bg-blue-500 hover:bg-blue-600 text-white text-xs font-bold transition-colors cursor-pointer shadow-md"
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}
