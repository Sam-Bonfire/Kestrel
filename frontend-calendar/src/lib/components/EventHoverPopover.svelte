<script lang="ts">
  import { Clock, MapPin, Video, Users, CheckCircle2, HelpCircle, XCircle } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import type { CalendarEvent } from './WeekGrid.svelte';

  let {
    event,
    anchor,
    onMouseEnter,
    onMouseLeave
  } = $props<{
    event: CalendarEvent | null;
    anchor: HTMLElement | null;
    onMouseEnter: () => void;
    onMouseLeave: () => void;
  }>();

  let popoverEl: HTMLElement | null = $state(null);
  let pos = $state({ top: 0, left: 0 });
  let visible = $state(false);

  // Format time (e.g. 10:00 AM)
  function formatTime(timeStr: string): string {
    const [h, m] = timeStr.split(':').map(Number);
    const date = new Date();
    date.setHours(h, m, 0, 0);
    return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
  }

  // Calculate duration string (e.g. 1h 30m)
  function getDuration(start: string, end: string): string {
    const [sh, sm] = start.split(':').map(Number);
    const [eh, em] = end.split(':').map(Number);
    const diffMins = (eh * 60 + em) - (sh * 60 + sm);

    if (diffMins <= 0) return '';

    const h = Math.floor(diffMins / 60);
    const m = diffMins % 60;

    if (h > 0 && m > 0) return `${h}h ${m}m`;
    if (h > 0) return `${h}h`;
    return `${m}m`;
  }

  const COLOR_CLASSES: Record<string, string> = {
    blue: 'bg-blue-500',
    purple: 'bg-purple-500',
    green: 'bg-emerald-500',
    orange: 'bg-orange-500',
    rose: 'bg-rose-500',
    amber: 'bg-amber-500',
    teal: 'bg-teal-500',
  };

  function getRSVPIcon(status: string | undefined) {
    if (status === 'yes') return CheckCircle2;
    if (status === 'no') return XCircle;
    return HelpCircle;
  }

  function getRSVPColor(status: string | undefined) {
    if (status === 'yes') return 'text-emerald-400';
    if (status === 'no') return 'text-rose-400';
    return 'text-amber-400';
  }

  let isVideoLink = $derived(event?.location?.startsWith('http://') || event?.location?.startsWith('https://'));

  $effect(() => {
    if (event && anchor) {
      // Need a tiny timeout to ensure popoverEl is rendered to measure it
      // but we initially position it hidden to get dimensions
      visible = true;

      const updatePosition = () => {
        if (!anchor || !popoverEl) return;

        const anchorRect = anchor.getBoundingClientRect();
        const popoverRect = popoverEl.getBoundingClientRect();

        const GAP = 8;

        // Preferred placement: right of the anchor
        let left = anchorRect.right + GAP;
        let top = anchorRect.top;

        // If it clips right edge, flip to left
        if (left + popoverRect.width > window.innerWidth) {
          left = anchorRect.left - popoverRect.width - GAP;
        }

        // If top + height clips bottom edge, push it up
        if (top + popoverRect.height > window.innerHeight) {
          top = Math.max(GAP, window.innerHeight - popoverRect.height - GAP);
        }

        // Month view case: anchors might be very wide, placing left/right might look weird or clip.
        // If anchor width is > 100px (likely month view), try placing it below or above instead.
        if (anchorRect.width > 100) {
          top = anchorRect.bottom + GAP;
          left = anchorRect.left + (anchorRect.width / 2) - (popoverRect.width / 2);

          // Clamp left/right
          if (left < GAP) left = GAP;
          if (left + popoverRect.width > window.innerWidth - GAP) {
             left = window.innerWidth - popoverRect.width - GAP;
          }

          // If clips bottom, place above
          if (top + popoverRect.height > window.innerHeight) {
             top = anchorRect.top - popoverRect.height - GAP;
          }
        }

        pos = { top, left };
      };

      // We use requestAnimationFrame to allow the DOM to update so popoverEl has dimensions
      requestAnimationFrame(() => {
        requestAnimationFrame(updatePosition);
      });
    } else {
      visible = false;
    }
  });

</script>

{#if event && anchor && visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={popoverEl}
    class="fixed z-[100] w-72 bg-[#181818] border border-neutral-800 rounded-xl shadow-2xl p-4 flex flex-col gap-3 font-sans"
    style="top: {pos.top}px; left: {pos.left}px;"
    transition:fade={{ duration: 150 }}
    onmouseenter={onMouseEnter}
    onmouseleave={onMouseLeave}
  >
    <!-- Header: Color + Title -->
    <div class="flex items-start gap-3">
      <div class="w-3 h-3 rounded-full mt-1.5 shrink-0 {COLOR_CLASSES[event.color] || COLOR_CLASSES.blue}"></div>
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-bold text-white leading-tight break-words">{event.title}</h3>
        <p class="text-[11px] text-[var(--color-text-secondary)] mt-0.5 font-mono">
          {formatTime(event.startTime)} - {formatTime(event.endTime)} ({getDuration(event.startTime, event.endTime)})
        </p>
      </div>
    </div>

    <!-- Location / Link -->
    {#if event.location}
      <div class="flex items-start gap-2 text-xs text-[var(--color-text-secondary)]">
        {#if isVideoLink}
          <Video class="w-4 h-4 shrink-0 mt-0.5" />
          <div class="flex-1 min-w-0 flex items-center justify-between gap-2">
            <span class="truncate">{event.location}</span>
            <a
              href={event.location}
              target="_blank"
              rel="noopener noreferrer"
              class="px-2 py-1 bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 hover:text-blue-300 rounded font-medium transition-colors shrink-0"
              onclick={(e) => e.stopPropagation()}
            >
              Join
            </a>
          </div>
        {:else}
          <MapPin class="w-4 h-4 shrink-0 mt-0.5" />
          <span class="truncate pt-0.5">{event.location}</span>
        {/if}
      </div>
    {/if}

    <!-- Attendees -->
    {#if event.attendees && event.attendees.length > 0}
      <div class="flex items-start gap-2 text-xs text-[var(--color-text-secondary)]">
        <Users class="w-4 h-4 shrink-0 mt-0.5" />
        <div class="flex-1 min-w-0 space-y-1 pt-0.5">
          <div class="font-medium text-white mb-1">{event.attendees.length} Attendees</div>
          <div class="flex flex-col gap-1 max-h-24 overflow-y-auto pr-1">
            {#each event.attendees as attendee}
              <div class="flex items-center justify-between">
                <span class="truncate max-w-[150px]">{attendee.name || attendee.email}</span>
                {#if attendee.rsvp}
                  {@const Icon = getRSVPIcon(attendee.rsvp)}
                  <Icon class="w-3.5 h-3.5 {getRSVPColor(attendee.rsvp)}" />
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Description snippet -->
    {#if event.description}
      <div class="text-[11px] text-[var(--color-text-secondary)]/80 line-clamp-3 bg-white/5 rounded-lg p-2 mt-1">
        {event.description}
      </div>
    {/if}

  </div>
{/if}
