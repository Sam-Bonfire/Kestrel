import { writable } from 'svelte/store';

// Location breadcrumb (K-1469): the bar shows where the user IS
// (`App / Current view`), derived from current state only. The previous
// recency-trail design rewrote itself on every tab/view switch and
// persisted across sessions, so the bar showed stale history instead of
// the current location.

/** Raw id of the currently displayed view ('inbox', '2-day', ...). */
export const currentCrumb = writable<string>('inbox');

export function setCurrentCrumb(label: string) {
  currentCrumb.set(label);
}

/** Turn a raw view id ('reply-later', 'label-receipts') into a display label. */
export function humanizeCrumb(id: string): string {
  const stripped = id.replace('label-', '');
  const spaced = stripped.replace(/-/g, ' ');
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}
