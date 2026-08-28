import { writable, get } from 'svelte/store';
import { getSettings, updateSettings } from '../api/client.js';

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
const SYNC_INTERVAL_KEY = 'kestrel:settings:sync_interval';
export const syncInterval = writable<number>(loadNumber(SYNC_INTERVAL_KEY, 300)); // Default 5 mins

let isInitializing = false;
let isUpdating = false;

export async function initializeSettings() {
  if (isInitializing) return;
  isInitializing = true;
  try {
    const settings = await getSettings();
    if (settings.mailDenseMode != null) mailDenseMode.set(settings.mailDenseMode);
    if (settings.mailDefaultLandingView != null) mailDefaultLandingView.set(settings.mailDefaultLandingView);
    if (settings.mailSignature != null) mailSignature.set(settings.mailSignature);
    if (settings.labelCustomizations != null) labelCustomizations.set(settings.labelCustomizations);
    if (settings.syncInterval != null) syncInterval.set(settings.syncInterval);

    // Also trigger snippet & signature template sync since we load settings together
    import('./templates.svelte.js').then((m) => {
      if (settings.snippets != null) {
        m.templateStore.snippets = settings.snippets;
        localStorage.setItem('kestrel:settings:snippets', JSON.stringify(settings.snippets));
      }
      if (settings.signatures != null) {
        m.templateStore.signatures = settings.signatures;
        localStorage.setItem('kestrel:settings:signatures', JSON.stringify(settings.signatures));
      }
    });

  } catch (err) {
    console.error('Failed to load settings from backend', err);
  } finally {
    isInitializing = false;
  }
}

async function syncToBackend() {
  if (isInitializing || isUpdating) return;
  // Only sync settings to backend if authenticated
  const { authState } = await import('./auth.svelte.js');
  if (!authState.isAuthenticated) return;

  isUpdating = true;
  try {
    await updateSettings({
      mailDenseMode: get(mailDenseMode),
      mailDefaultLandingView: get(mailDefaultLandingView),
      mailSignature: get(mailSignature),
      labelCustomizations: get(labelCustomizations),
      syncInterval: get(syncInterval),
      // we'll update theme too if available
      theme: localStorage.getItem('kestrel:settings:theme') || 'system',
    });
  } catch (err) {
    console.error('Failed to sync settings to backend', err);
  } finally {
    isUpdating = false;
  }
}

// Subscribe & persist settings changes
mailDenseMode.subscribe((val) => {
  saveItem(DENSE_KEY, String(val));
  syncToBackend();
});
mailDefaultLandingView.subscribe((val) => {
  saveItem(LANDING_KEY, val);
  syncToBackend();
});
mailSignature.subscribe((val) => {
  saveItem(SIG_KEY, val);
  syncToBackend();
});
labelCustomizations.subscribe((val) => {
  saveItem(LABELS_KEY, JSON.stringify(val));
  syncToBackend();
});
syncInterval.subscribe((val) => {
  saveItem(SYNC_INTERVAL_KEY, String(val));
  syncToBackend();
});

function loadNumber(key: string, def: number): number {
  try {
    const val = localStorage.getItem(key);
    return val !== null ? Number(val) : def;
  } catch {
    return def;
  }
}

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
const THEME_KEY = 'kestrel:settings:theme';
export const theme = writable<string>(loadStr(THEME_KEY, 'system'));
theme.subscribe((val) => {
  saveItem(THEME_KEY, val);
  syncToBackend();
});
