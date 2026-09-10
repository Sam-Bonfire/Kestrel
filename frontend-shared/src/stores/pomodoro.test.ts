import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  pomodoro,
  pomodoroDurations,
  startPomodoro,
  pausePomodoro,
  resetPomodoro,
  skipPomodoroPhase,
  tickPomodoro,
  stopPomodoro,
  setPomodoroCompleteHandler,
} from './pomodoro.js';

const mockStorage: Record<string, string> = {};
globalThis.localStorage = {
  getItem: (key: string) => mockStorage[key] ?? null,
  setItem: (key: string, value: string) => { mockStorage[key] = value; },
  removeItem: (key: string) => { delete mockStorage[key]; },
  clear: () => { Object.keys(mockStorage).forEach(k => delete mockStorage[k]); },
  length: 0,
  key: () => null,
};

describe('Pomodoro timer', () => {
  beforeEach(() => {
    stopPomodoro();
    setPomodoroCompleteHandler(null);
    pomodoroDurations.set({ work: 25 * 60, shortBreak: 5 * 60, longBreak: 15 * 60 });
    pomodoro.set({ phase: 'work', status: 'idle', secondsLeft: 25 * 60, completedWorkSessions: 0 });
  });

  it('starts and ticks down', () => {
    startPomodoro();
    expect(get(pomodoro).status).toBe('running');
    const before = get(pomodoro).secondsLeft;
    tickPomodoro();
    expect(get(pomodoro).secondsLeft).toBe(before - 1);
    stopPomodoro();
  });

  it('pauses and resumes without losing time', () => {
    startPomodoro();
    tickPomodoro();
    pausePomodoro();
    const frozen = get(pomodoro).secondsLeft;
    tickPomodoro();
    expect(get(pomodoro).secondsLeft).toBe(frozen);
    expect(get(pomodoro).status).toBe('paused');
    startPomodoro();
    expect(get(pomodoro).status).toBe('running');
    stopPomodoro();
  });

  it('advances work to short break and notifies', () => {
    const finished: string[] = [];
    setPomodoroCompleteHandler((phase) => finished.push(phase));
    pomodoroDurations.set({ work: 2, shortBreak: 5 * 60, longBreak: 15 * 60 });
    resetPomodoro();
    startPomodoro();
    tickPomodoro();
    tickPomodoro();
    const state = get(pomodoro);
    expect(state.phase).toBe('shortBreak');
    expect(state.status).toBe('idle');
    expect(state.completedWorkSessions).toBe(1);
    expect(finished).toEqual(['work']);
    stopPomodoro();
  });

  it('takes a long break every fourth work session', () => {
    pomodoroDurations.set({ work: 1, shortBreak: 1, longBreak: 30 });
    resetPomodoro();
    for (let i = 0; i < 3; i++) {
      startPomodoro();
      tickPomodoro();
      expect(get(pomodoro).phase).toBe('shortBreak');
      skipPomodoroPhase(); // back to work, session count kept
    }
    startPomodoro();
    tickPomodoro();
    expect(get(pomodoro).phase).toBe('longBreak');
    expect(get(pomodoro).completedWorkSessions).toBe(4);
    stopPomodoro();
  });

  it('skips without counting and resets the clock', () => {
    startPomodoro();
    skipPomodoroPhase();
    const state = get(pomodoro);
    expect(state.phase).toBe('shortBreak');
    expect(state.completedWorkSessions).toBe(0);
    expect(state.status).toBe('idle');
  });

  it('persists custom durations', () => {
    pomodoroDurations.set({ work: 50 * 60, shortBreak: 10 * 60, longBreak: 20 * 60 });
    expect(localStorage.getItem('kestrel:pomodoro:durations')).toContain('3000');
  });
});
