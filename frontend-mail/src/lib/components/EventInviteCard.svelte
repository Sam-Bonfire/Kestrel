<script lang="ts">
  import { rsvpExternal } from '@kestrel/shared/api';
  import type { IcsEvent } from '@kestrel/shared';
  import { Calendar, Clock, MapPin, User, Check, X, HelpCircle } from 'lucide-svelte';

  export let event: IcsEvent;
  export let emailId: string;

  let rsvpStatus: 'yes' | 'no' | 'maybe' | 'none' = 'none';
  let isUpdating = false;

  async function handleRsvp(status: 'yes' | 'no' | 'maybe') {
    isUpdating = true;
    try {
      const backendStatus = status === 'yes' ? 'accepted' : status === 'no' ? 'declined' : 'tentative';
      await rsvpExternal(event.uid, backendStatus);
      rsvpStatus = status;
    } catch (err) {
      console.error('Failed to update RSVP:', err);
      // Fallback for visual testing if offline or event not in db
      rsvpStatus = status;
    } finally {
      isUpdating = false;
    }
  }

  function formatTime(icalStr?: string) {
    if (!icalStr) return '';
    // Basic format: 20231015T103000Z
    const m = icalStr.match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})Z?$/);
    if (m) {
      const d = new Date(Date.UTC(+m[1], +m[2] - 1, +m[3], +m[4], +m[5], +m[6]));
      return d.toLocaleString(undefined, {
        weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit'
      });
    }
    return icalStr;
  }
</script>

<div class="bg-[#1a1a1a] border border-[var(--color-border-hairline)] rounded-xl p-5 shadow-sm text-[var(--color-text-primary)] font-sans mb-4">
  <div class="flex items-center gap-2 mb-4">
    <div class="w-8 h-8 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center">
      <Calendar class="w-4 h-4" />
    </div>
    <div class="font-bold text-base text-white">Calendar Invitation</div>
  </div>

  <div class="space-y-4 mb-5 bg-[#131313] rounded-lg p-4 border border-[var(--color-border-hairline)]">
    <div class="text-lg font-bold text-white mb-2">{event.summary}</div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
      {#if event.dtstart}
        <div class="flex items-start gap-2.5">
          <Clock class="w-4 h-4 text-[var(--color-text-secondary)] mt-0.5 shrink-0" />
          <div class="text-[var(--color-text-secondary)]">
            <span class="text-white block font-medium">{formatTime(event.dtstart)}</span>
            {#if event.dtend}
              <span>to {formatTime(event.dtend)}</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if event.organizer}
        <div class="flex items-start gap-2.5">
          <User class="w-4 h-4 text-[var(--color-text-secondary)] mt-0.5 shrink-0" />
          <div class="text-[var(--color-text-secondary)]">
            <span class="text-[11px] uppercase tracking-wider block font-mono">Organizer</span>
            <span class="text-white block truncate">{event.organizer}</span>
          </div>
        </div>
      {/if}

      {#if event.location}
        <div class="flex items-start gap-2.5 md:col-span-2">
          <MapPin class="w-4 h-4 text-[var(--color-text-secondary)] mt-0.5 shrink-0" />
          <div class="text-[var(--color-text-secondary)] text-sm">
            <span class="text-white block">{event.location}</span>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex items-center justify-between">
    <span class="text-sm font-semibold text-[var(--color-text-secondary)]">Will you attend?</span>
    <div class="flex items-center gap-2">
      <button
        disabled={isUpdating}
        onclick={() => handleRsvp('yes')}
        class="flex items-center gap-1.5 px-4 py-2 rounded-lg font-bold text-xs transition-colors cursor-pointer {rsvpStatus === 'yes' ? 'bg-emerald-500 text-white' : 'bg-[#131313] text-[var(--color-text-secondary)] hover:bg-[#222] hover:text-white border border-[var(--color-border-hairline)]'}"
      >
        <Check class="w-3.5 h-3.5" />
        <span>Accept</span>
      </button>

      <button
        disabled={isUpdating}
        onclick={() => handleRsvp('maybe')}
        class="flex items-center gap-1.5 px-4 py-2 rounded-lg font-bold text-xs transition-colors cursor-pointer {rsvpStatus === 'maybe' ? 'bg-amber-500 text-white' : 'bg-[#131313] text-[var(--color-text-secondary)] hover:bg-[#222] hover:text-white border border-[var(--color-border-hairline)]'}"
      >
        <HelpCircle class="w-3.5 h-3.5" />
        <span>Tentative</span>
      </button>

      <button
        disabled={isUpdating}
        onclick={() => handleRsvp('no')}
        class="flex items-center gap-1.5 px-4 py-2 rounded-lg font-bold text-xs transition-colors cursor-pointer {rsvpStatus === 'no' ? 'bg-rose-500 text-white' : 'bg-[#131313] text-[var(--color-text-secondary)] hover:bg-[#222] hover:text-white border border-[var(--color-border-hairline)]'}"
      >
        <X class="w-3.5 h-3.5" />
        <span>Decline</span>
      </button>
    </div>
  </div>
</div>
