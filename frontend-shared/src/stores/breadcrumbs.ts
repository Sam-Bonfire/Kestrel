import { writable } from 'svelte/store';

export interface Breadcrumb {
  label: string;
}

export const MAX_CRUMBS = 8;

const CRUMBS_KEY = 'kestrel:nav:recent';

function loadCrumbs(): Breadcrumb[] {
  try {
    if (typeof localStorage !== 'undefined') {
      const val = localStorage.getItem(CRUMBS_KEY);
      const parsed = val !== null ? JSON.parse(val) : [];
      return Array.isArray(parsed) ? parsed : [];
    }
    return [];
  } catch {
    return [];
  }
}

/** Newest-first trail of recently visited views. */
export const recentBreadcrumbs = writable<Breadcrumb[]>(loadCrumbs());

recentBreadcrumbs.subscribe((val) => {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(CRUMBS_KEY, JSON.stringify(val));
    }
  } catch {
    // Non-fatal
  }
});

export function pushBreadcrumb(label: string) {
  recentBreadcrumbs.update((crumbs) => {
    const rest = crumbs.filter((c) => c.label !== label);
    return [{ label }, ...rest].slice(0, MAX_CRUMBS);
  });
}

export function clearBreadcrumbs() {
  recentBreadcrumbs.set([]);
}

/** Turn a raw view id ('reply-later', 'label-receipts') into a display label. */
export function humanizeCrumb(id: string): string {
  const stripped = id.replace('label-', '');
  const spaced = stripped.replace(/-/g, ' ');
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}
