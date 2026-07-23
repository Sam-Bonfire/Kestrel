<script lang="ts">
  import { X, Calendar as CalendarIcon, Clock, MapPin, AlignLeft, Users, Bell, Flag, Check } from 'lucide-svelte';

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
    selectedDateStr = ''
  } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
    onSave?: (eventData: any) => void;
    selectedDateStr?: string;
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
  let attendeesInput = $state('');

  // Hydrate date when modal opens
  $effect(() => {
    if (isOpen) {
      date = selectedDateStr || new Date().toISOString().split('T')[0];
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
      rsvpStatus,
      attendees: attendeesInput.split(',').map(email => email.trim()).filter(Boolean)
    });

    // Reset
    title = '';
    description = '';
    location = '';
    priority = 'None';
    status = 'Scheduled';
    category = 'Work';
    rsvpStatus = 'none';
    attendeesInput = '';
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
      class="w-full max-w-xl bg-[#131313] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl flex flex-col overflow-hidden text-xs text-[var(--color-text-primary)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-4 py-3.5 border-b border-[var(--color-border-hairline)] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2">
          <CalendarIcon class="w-4 h-4 text-blue-400" />
          <span class="font-bold text-white uppercase tracking-wider">Schedule Event</span>
        </div>
        <button onclick={onClose} class="p-1 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Scrollable form inputs -->
      <div class="p-6 space-y-4 max-h-[70vh] overflow-y-auto">
        <!-- Title -->
        <div class="space-y-1">
          <label for="title" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Event Title</label>
          <input
            id="title"
            type="text"
            placeholder="Review Spec Architecture..."
            bind:value={title}
            class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] focus:border-white/20 rounded-lg p-2.5 outline-none text-white transition-colors"
            required
          />
        </div>

        <!-- Date & Time Row -->
        <div class="grid grid-cols-3 gap-3">
          <div class="space-y-1 relative">
            <label for="date" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Date</label>
            <input
              id="date"
              type="date"
              bind:value={date}
              style="color-scheme: dark;"
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] focus:border-white/20 focus:ring-1 focus:ring-white/20 rounded-lg p-2 outline-none text-white transition-all hover:bg-[var(--color-canvas-hover)] cursor-pointer"
            />
          </div>
          <div class="space-y-1 relative">
            <label for="start" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Start Time</label>
            <input
              id="start"
              type="time"
              bind:value={startTime}
              style="color-scheme: dark;"
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] focus:border-white/20 focus:ring-1 focus:ring-white/20 rounded-lg p-2 outline-none text-white transition-all hover:bg-[var(--color-canvas-hover)] cursor-pointer"
            />
          </div>
          <div class="space-y-1 relative">
            <label for="end" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">End Time</label>
            <input
              id="end"
              type="time"
              bind:value={endTime}
              style="color-scheme: dark;"
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] focus:border-white/20 focus:ring-1 focus:ring-white/20 rounded-lg p-2 outline-none text-white transition-all hover:bg-[var(--color-canvas-hover)] cursor-pointer"
            />
          </div>
        </div>

        <!-- Location -->
        <div class="space-y-1">
          <label for="location" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Location / Conference Link</label>
          <input
            id="location"
            type="text"
            placeholder="Gather Town Workspace or Zoom URL..."
            bind:value={location}
            class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2.5 outline-none text-white transition-colors"
          />
        </div>

        <!-- Description -->
        <div class="space-y-1">
          <label for="desc" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Description</label>
          <textarea
            id="desc"
            placeholder="Provide context for this session..."
            bind:value={description}
            rows="3"
            class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2.5 outline-none text-white transition-colors resize-none"
          ></textarea>
        </div>

        <!-- Categories & Priority Row -->
        <div class="grid grid-cols-3 gap-3">
          <div class="space-y-1">
            <label for="category" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Category</label>
            <select
              id="category"
              bind:value={category}
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2 outline-none text-white"
            >
              <option value="Work">Work</option>
              <option value="Personal">Personal</option>
              <option value="Workspace">Workspace</option>
              <option value="General">General</option>
            </select>
          </div>

          <div class="space-y-1">
            <label for="priority" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Priority</label>
            <select
              id="priority"
              bind:value={priority}
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2 outline-none text-white"
            >
              <option value="High">🔴 High</option>
              <option value="Medium">🟡 Medium</option>
              <option value="Low">🔵 Low</option>
              <option value="None">None</option>
            </select>
          </div>

          <div class="space-y-1">
            <label for="status" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Status</label>
            <select
              id="status"
              bind:value={status}
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2 outline-none text-white"
            >
              <option value="Scheduled">Scheduled</option>
              <option value="Next Up">Next Up</option>
              <option value="In Progress">In Progress</option>
              <option value="Completed">Completed</option>
            </select>
          </div>
        </div>

        <!-- RSVP & Attendees -->
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label for="rsvp" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">My RSVP Status</label>
            <select
              id="rsvp"
              bind:value={rsvpStatus}
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2 outline-none text-white"
            >
              <option value="none">Choose Response...</option>
              <option value="yes">Yes, attending</option>
              <option value="no">No, declining</option>
              <option value="maybe">Maybe</option>
            </select>
          </div>

          <div class="space-y-1">
            <label for="attendees" class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Attendees (comma-separated)</label>
            <input
              id="attendees"
              type="text"
              placeholder="sam@kestrel.dev, alex@gmail.com"
              bind:value={attendeesInput}
              class="w-full bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg p-2.5 outline-none text-white transition-colors"
            />
          </div>
        </div>

        <!-- Color Selector Dots -->
        <div class="space-y-1.5 pt-1">
          <span class="block font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">Display Theme Color</span>
          <div class="flex items-center gap-2">
            {#each COLOR_OPTIONS as col}
              <button
                onclick={() => color = col.name}
                class="w-5 h-5 rounded-full {col.dot} relative cursor-pointer transition-transform hover:scale-110"
                style="box-shadow: {color === col.name ? '0 0 6px rgba(255,255,255,0.6)' : 'none'}"
                title={col.name}
              >
                {#if color === col.name}
                  <span class="absolute inset-0 flex items-center justify-center">
                    <span class="w-1 h-1 rounded-full bg-white/70"></span>
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="px-6 py-3.5 border-t border-[var(--color-border-hairline)] flex items-center justify-end gap-2.5 bg-[#191919]">
        <button 
          onclick={onClose} 
          class="px-3 py-1.5 rounded text-[11px] font-medium text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
        >
          Cancel
        </button>
        <button 
          onclick={handleSave} 
          class="px-4 py-1.5 rounded bg-white text-black text-[11px] font-semibold hover:bg-neutral-200 transition-colors cursor-pointer shadow-md"
        >
          Save Event
        </button>
      </div>
    </div>
  </div>
{/if}
