import { writable, get } from 'svelte/store';

export type PomodoroPhase = 'work' | 'shortBreak' | 'longBreak';
export type PomodoroStatus = 'idle' | 'running' | 'paused';

export interface PomodoroState {
  phase: PomodoroPhase;
  status: PomodoroStatus;
  secondsLeft: number;
  completedWorkSessions: number;
}

export interface PomodoroDurations {
  work: number;
  shortBreak: number;
  longBreak: number;
}

const DURATIONS_KEY = 'kestrel:pomodoro:durations';
const DEFAULT_DURATIONS: PomodoroDurations = { work: 25 * 60, shortBreak: 5 * 60, longBreak: 15 * 60 };
const SESSIONS_BEFORE_LONG = 4;

function loadDurations(): PomodoroDurations {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(DURATIONS_KEY);
      if (raw) {
        const parsed = JSON.parse(raw);
        return {
          work: positiveOr(parsed.work, DEFAULT_DURATIONS.work),
          shortBreak: positiveOr(parsed.shortBreak, DEFAULT_DURATIONS.shortBreak),
          longBreak: positiveOr(parsed.longBreak, DEFAULT_DURATIONS.longBreak),
        };
      }
    }
  } catch {
    // Non-fatal
  }
  return { ...DEFAULT_DURATIONS };
}

function positiveOr(value: unknown, fallback: number): number {
  return typeof value === 'number' && value > 0 ? Math.floor(value) : fallback;
}

export const pomodoroDurations = writable<PomodoroDurations>(loadDurations());

pomodoroDurations.subscribe((val) => {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(DURATIONS_KEY, JSON.stringify(val));
    }
  } catch {
    // Non-fatal
  }
});

function freshState(durations: PomodoroDurations): PomodoroState {
  return { phase: 'work', status: 'idle', secondsLeft: durations.work, completedWorkSessions: 0 };
}

export const pomodoro = writable<PomodoroState>(freshState(get(pomodoroDurations)));

type PhaseHandler = (finished: PomodoroPhase) => void;
let onPhaseComplete: PhaseHandler | null = null;

/** Register a one-way notification hook (toast, OS notification). */
export function setPomodoroCompleteHandler(handler: PhaseHandler | null) {
  onPhaseComplete = handler;
}

let interval: ReturnType<typeof setInterval> | null = null;
function stopTick() {
  if (interval !== null) {
    clearInterval(interval);
    interval = null;
  }
}

function startTick() {
  stopTick();
  // ponytail: plain 1s decrement drifts under background throttle;
  // switch to a Date.now() deadline if inaccuracy is ever reported.
  interval = setInterval(() => tickPomodoro(), 1000);
}

export function startPomodoro() {
  const state = get(pomodoro);
  if (state.status === 'running') return;
  // Restart a finished session instead of resuming at zero.
  if (state.secondsLeft <= 0) {
    pomodoro.set({ ...state, secondsLeft: phaseDuration(state.phase), status: 'running' });
  } else {
    pomodoro.set({ ...state, status: 'running' });
  }
  startTick();
}

export function pausePomodoro() {
  stopTick();
  pomodoro.update((s) => (s.status === 'running' ? { ...s, status: 'paused' } : s));
}

export function resetPomodoro() {
  stopTick();
  pomodoro.update((s) => ({ ...s, secondsLeft: phaseDuration(s.phase), status: 'idle' }));
}

/** Skip to the next phase without counting the current one. */
export function skipPomodoroPhase() {
  stopTick();
  pomodoro.update((s) => {
    const next = nextPhase(s.phase, s.completedWorkSessions);
    return { ...s, phase: next, secondsLeft: phaseDuration(next), status: 'idle' };
  });
}

/** Advance one second; exported for tests (the interval calls it). */
export function tickPomodoro() {
  const s = get(pomodoro);
  if (s.status !== 'running') return;
  if (s.secondsLeft > 1) {
    pomodoro.set({ ...s, secondsLeft: s.secondsLeft - 1 });
    return;
  }
  // Phase finished.
  stopTick();
  const completed = s.phase === 'work' ? s.completedWorkSessions + 1 : s.completedWorkSessions;
  const next = nextPhase(s.phase, completed);
  pomodoro.set({ phase: next, status: 'idle', secondsLeft: phaseDuration(next), completedWorkSessions: completed });
  onPhaseComplete?.(s.phase);
}

/** Stop the timer entirely (tests, unmount). */
export function stopPomodoro() {
  stopTick();
  pomodoro.update((s) => ({ ...s, status: 'idle' }));
}

function nextPhase(phase: PomodoroPhase, completedWorkSessions: number): PomodoroPhase {
  if (phase !== 'work') return 'work';
  return completedWorkSessions > 0 && completedWorkSessions % SESSIONS_BEFORE_LONG === 0
    ? 'longBreak'
    : 'shortBreak';
}

function phaseDuration(phase: PomodoroPhase): number {
  const durations = get(pomodoroDurations);
  return phase === 'work' ? durations.work : phase === 'shortBreak' ? durations.shortBreak : durations.longBreak;
}
