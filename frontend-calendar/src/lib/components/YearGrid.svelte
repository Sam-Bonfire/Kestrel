<script lang="ts">
  import { CalendarDays } from 'lucide-svelte';
  import { buildEventDensityMap, daysInMonth, toISODateString } from '@kestrel/shared';
  import type { CalendarEvent } from './WeekGrid.svelte';

  let {
    events = [] as CalendarEvent[],
    selectedDate = new Date(),
    onSelectDate = (_d: Date) => {},
    onChangeViewMode = (_v: string) => {},
  } = $props<{
    events?: CalendarEvent[];
    selectedDate?: Date;
    onSelectDate?: (date: Date) => void;
    onChangeViewMode?: (v: string) => void;
  }>();

  const year = $derived(selectedDate.getFullYear());
  const density = $derived(buildEventDensityMap(events));
  const todayStr = toISODateString(
    new Date().getFullYear(),
    new Date().getMonth(),
    new Date().getDate()
  );

  const MONTH_NAMES = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December',
  ];

  function monthCells(y: number, m: number): (number | null)[] {
    // Monday-first offset
    const offset = (new Date(y, m, 1).getDay() + 6) % 7;
    const cells: (number | null)[] = [];
    for (let i = 0; i < offset; i++) cells.push(null);
    for (let d = 1; d <= daysInMonth(y, m); d++) cells.push(d);
    return cells;
  }

  function densityClass(count: number): string {
    if (count >= 6) return 'bg-blue-500/75 text-white font-medium';
    if (count >= 3) return 'bg-blue-500/45 text-white';
    if (count >= 1) return 'bg-blue-500/20 text-blue-100';
    return 'text-[var(--color-text-secondary)] hover:bg-white/5';
  }
</script>

<div class="flex-1 overflow-y-auto p-4 md:p-6" role="region" aria-label="Year at a glance">
  <div class="grid grid-cols-1 md:grid-cols-3 xl:grid-cols-4 gap-4">
    {#each MONTH_NAMES as name, m}
      <section aria-label="{name} {year}" class="rounded-xl border border-[var(--color-border-hairline)] bg-[var(--color-canvas-card)] p-3">
        <h3 class="flex items-center gap-1.5 text-xs font-semibold text-white mb-2">
          <CalendarDays class="w-3.5 h-3.5 text-[var(--color-text-secondary)]" />
          {name}
        </h3>
        <div class="grid grid-cols-7 gap-0.5 text-center text-[10px] font-mono text-[var(--color-text-secondary)]/60 mb-1">
          {#each ['M', 'T', 'W', 'T', 'F', 'S', 'S'] as wd}
            <span>{wd}</span>
          {/each}
        </div>
        <div class="grid grid-cols-7 gap-0.5">
          {#each monthCells(year, m) as day}
            {#if day === null}
              <span></span>
            {:else}
              {@const iso = toISODateString(year, m, day)}
              {@const info = density[iso]}
              {@const count = info?.count ?? 0}
              <div class="group relative">
                <button
                  type="button"
                  onclick={() => { onSelectDate(new Date(year, m, day)); onChangeViewMode('day'); }}
                  aria-label="{iso}, {count} events"
                  class="w-full aspect-square rounded text-[11px] font-mono transition-colors cursor-pointer {densityClass(count)} {iso === todayStr ? 'ring-2 ring-blue-500' : ''}"
                >
                  {day}
                </button>
                {#if count > 0}
                  <div class="hidden group-hover:block absolute z-30 bottom-full left-1/2 -translate-x-1/2 mb-1 w-40 rounded-lg border border-neutral-700 bg-[#1a1a1a] p-2 text-left shadow-2xl pointer-events-none">
                    <div class="text-[11px] font-semibold text-white">{iso} · {count} event{count === 1 ? '' : 's'}</div>
                    {#each info?.titles ?? [] as t}
                      <div class="text-[10px] text-neutral-400 truncate">{t}</div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          {/each}
        </div>
      </section>
    {/each}
  </div>
</div>
