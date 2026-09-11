<script lang="ts">
  import { Play, Pause, SkipForward, RotateCcw, Timer } from 'lucide-svelte';
  import {
    pomodoro,
    startPomodoro,
    pausePomodoro,
    resetPomodoro,
    skipPomodoroPhase,
  } from '../stores/pomodoro.js';

  const phaseLabels = { work: 'Focus', shortBreak: 'Short break', longBreak: 'Long break' } as const;

  function format(totalSeconds: number): string {
    const m = Math.floor(totalSeconds / 60);
    const s = totalSeconds % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
</script>

<div class="px-2.5 py-2 rounded-lg bg-white/[0.03] border border-white/5" aria-label="Pomodoro focus timer">
  <div class="flex items-center justify-between">
    <span class="flex items-center gap-1.5 text-[11px] font-semibold text-[var(--color-text-secondary)]">
      <Timer class="w-3.5 h-3.5" />
      {phaseLabels[$pomodoro.phase]}
    </span>
    <span class="text-sm font-mono font-bold text-white tabular-nums" aria-live="polite">
      {format($pomodoro.secondsLeft)}
    </span>
  </div>
  <div class="flex items-center gap-1 mt-1.5">
    {#if $pomodoro.status === 'running'}
      <button type="button" onclick={pausePomodoro} title="Pause" aria-label="Pause timer" class="p-1.5 rounded-md hover:bg-white/10 text-white transition-colors cursor-pointer">
        <Pause class="w-3.5 h-3.5" />
      </button>
    {:else}
      <button type="button" onclick={startPomodoro} title="Start" aria-label="Start timer" class="p-1.5 rounded-md hover:bg-white/10 text-white transition-colors cursor-pointer">
        <Play class="w-3.5 h-3.5" />
      </button>
    {/if}
    <button type="button" onclick={skipPomodoroPhase} title="Skip to next phase" aria-label="Skip to next phase" class="p-1.5 rounded-md hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer">
      <SkipForward class="w-3.5 h-3.5" />
    </button>
    <button type="button" onclick={resetPomodoro} title="Reset timer" aria-label="Reset timer" class="p-1.5 rounded-md hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer">
      <RotateCcw class="w-3.5 h-3.5" />
    </button>
    <span class="ml-auto text-[10px] font-mono text-[var(--color-text-secondary)]/70" title="Completed work sessions">
      {$pomodoro.completedWorkSessions} done
    </span>
  </div>
</div>
