import { writable } from 'svelte/store';

const DENSE_KEY = 'kestrel:settings:dense_mode';
const LANDING_KEY = 'kestrel:settings:landing_view';
const SIG_KEY = 'kestrel:settings:signature';
const LABELS_KEY = 'kestrel:settings:label_customizations';

export const mailDenseMode = writable<boolean>(loadBool(DENSE_KEY, false));
export const mailDefaultLandingView = writable<string>(loadStr(LANDING_KEY, 'inbox'));
export const mailSignature = writable<string>(loadStr(SIG_KEY, ''));
export const labelCustomizations = writable<Record<string, { iconName: string; colorName: string }>>(
  loadJson(LABELS_KEY, {})
);

// Subscribe & persist settings changes
mailDenseMode.subscribe((val) => saveItem(DENSE_KEY, String(val)));
mailDefaultLandingView.subscribe((val) => saveItem(LANDING_KEY, val));
mailSignature.subscribe((val) => saveItem(SIG_KEY, val));
labelCustomizations.subscribe((val) => saveItem(LABELS_KEY, JSON.stringify(val)));

function loadBool(key: string, def: boolean): boolean {
  try {
    const val = localStorage.getItem(key);
    return val !== null ? val === 'true' : def;
  } catch {
    return def;
  }
}

function loadStr(key: string, def: string): string {
  try {
    const val = localStorage.getItem(key);
    return val !== null ? val : def;
  } catch {
    return def;
  }
}

function loadJson<T>(key: string, def: T): T {
  try {
    const val = localStorage.getItem(key);
    return val !== null ? JSON.parse(val) : def;
  } catch {
    return def;
  }
}

function saveItem(key: string, val: string): void {
  try {
    localStorage.setItem(key, val);
  } catch {
    // Non-fatal
  }
}
