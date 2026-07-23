<script lang="ts">
  import type { CalendarEvent } from './WeekGrid.svelte';

  let {
    events = [],
    onEventClick = (ev: CalendarEvent) => {}
  } = $props<{
    events?: CalendarEvent[];
    onEventClick?: (ev: CalendarEvent) => void;
  }>();

  const daysOfWeek = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const monthDays = Array.from({ length: 35 }, (_, i) => i + 1);
</script>

<div class="flex-1 h-screen bg-[var(--color-canvas-base)] flex flex-col font-sans overflow-hidden">
  <!-- Month Header -->
  <div class="grid grid-cols-7 border-b border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)] text-xs font-semibold text-[var(--color-text-secondary)]">
    {#each daysOfWeek as day}
      <div class="p-3 text-center border-r border-[var(--color-border-hairline)] last:border-r-0">
        {day}
      </div>
    {/each}
  </div>

  <!-- Month Grid -->
  <div class="flex-1 grid grid-cols-7 grid-rows-5 divide-x divide-y divide-[var(--color-border-hairline)]/30 overflow-hidden">
    {#each monthDays as dayNum, i}
      <div class="p-2 border-r border-b border-[var(--color-border-hairline)]/30 flex flex-col justify-between hover:bg-[var(--color-canvas-hover)]/20 transition-colors">
        <div class="text-[11px] font-mono text-[var(--color-text-secondary)] font-semibold mb-1">
          {dayNum <= 31 ? dayNum : dayNum - 31}
        </div>

        <div class="flex-1 space-y-1 overflow-y-auto">
          {#each events.filter(e => e.dayIndex === (i % 7)) as ev}
            <button
              onclick={() => onEventClick(ev)}
              class="w-full px-2 py-1 rounded text-[10px] font-medium text-white text-left truncate shadow-sm cursor-pointer"
              style="background-color: {ev.color};"
            >
              {ev.title}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>
