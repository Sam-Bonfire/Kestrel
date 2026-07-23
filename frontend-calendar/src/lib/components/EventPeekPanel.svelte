<script lang="ts">
  import { Avatar, LabelPill, Button } from '@kestrel/shared';
  import { X, Clock, MapPin, Users, Edit, Trash2, Calendar as CalendarIcon } from 'lucide-svelte';

  export interface EventDetail {
    id: string;
    title: string;
    date: string;
    startTime: string;
    endTime: string;
    description?: string;
    color: string;
    location?: string;
    category?: string;
    organizer?: string;
  }

  let {
    event = null,
    onClose = () => {},
    onEdit = () => {},
    onDelete = () => {}
  } = $props<{
    event?: EventDetail | null;
    clickPosition?: { x: number, y: number } | null;
    onClose?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
  }>();
  
  let popoverStyle = $derived(() => {
    if (!clickPosition) return 'top: 50%; left: 50%; transform: translate(-50%, -50%);';
    // Position near the click, keeping it on screen (assuming ~350px width and ~300px height)
    let x = clickPosition.x + 20;
    let y = clickPosition.y - 20;
    if (typeof window !== 'undefined') {
      if (x + 350 > window.innerWidth) x = window.innerWidth - 370;
      if (y + 300 > window.innerHeight) y = window.innerHeight - 320;
    }
    return `top: ${Math.max(10, y)}px; left: ${Math.max(10, x)}px;`;
  });
</script>

{#if event}
  <!-- Invisible backdrop to catch clicks outside the popover -->
  <div
    id="event-details-overlay"
    class="fixed inset-0 z-40 bg-transparent"
    role="button"
    tabindex="0"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  ></div>

  <!-- Contextual Popover -->
  <div
    id="event-details-modal"
    class="fixed z-50 w-[350px] bg-[#131313] border border-[var(--color-border-hairline)] rounded-xl shadow-[0_8px_30px_rgba(0,0,0,0.5)] overflow-hidden font-sans animate-fadeIn origin-top-left"
    style={popoverStyle()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 bg-[#181818] border-b border-[var(--color-border-hairline)] flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-3 h-3 rounded-full" style="background-color: {event.color};"></div>
          <LabelPill tag="urgent" label={event.category || 'Event Details'} />
        </div>

        <div class="flex items-center gap-1">
          <button onclick={onEdit} title="Edit Event" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white">
            <Edit class="w-4 h-4" />
          </button>
          <button onclick={onDelete} title="Delete Event" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-red-400">
            <Trash2 class="w-4 h-4" />
          </button>
          <button onclick={onClose} title="Close" class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Content Body -->
      <div class="p-6 space-y-5">
        <h2 class="text-xl font-bold text-white leading-snug">{event.title}</h2>

        <div class="space-y-3 text-xs text-[var(--color-text-secondary)]">
          <div class="flex items-center gap-3">
            <CalendarIcon class="w-4 h-4 text-blue-400 shrink-0" />
            <span class="text-white font-medium">{event.date}</span>
          </div>

          <div class="flex items-center gap-3">
            <Clock class="w-4 h-4 text-amber-400 shrink-0" />
            <span>{event.startTime} - {event.endTime}</span>
          </div>

          {#if event.location}
            <div class="flex items-center gap-3">
              <MapPin class="w-4 h-4 text-emerald-400 shrink-0" />
              <span class="text-white">{event.location}</span>
            </div>
          {/if}

          {#if event.organizer}
            <div class="flex items-center gap-3 min-w-0">
              <Users class="w-4 h-4 text-violet-400 shrink-0" />
              <span class="truncate flex items-center gap-2">
                <span>Organized by</span>
                <Avatar name={event.organizer} size="sm" />
                <strong class="text-white">{event.organizer}</strong>
              </span>
            </div>
          {/if}
        </div>

        {#if event.description}
          <div class="pt-3 border-t border-[var(--color-border-hairline)] text-xs text-[var(--color-text-primary)] leading-relaxed font-sans">
            {event.description}
          </div>
        {/if}
      </div>

      <!-- Action Footer using @kestrel/shared Button -->
      <div class="px-5 py-3 bg-[#181818] border-t border-[var(--color-border-hairline)] flex items-center justify-end">
        <Button variant="primary" size="sm" onClick={onClose}>
          Close
        </Button>
      </div>
    </div>
{/if}
