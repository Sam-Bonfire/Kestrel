import { readable } from 'svelte/store';

export const relativeTimeTick = readable<number>(Date.now(), (set) => {
  const interval = setInterval(() => {
    set(Date.now());
  }, 60000);
  return () => clearInterval(interval);
});
