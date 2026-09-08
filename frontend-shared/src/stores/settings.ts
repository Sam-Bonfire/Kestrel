import { writable, derived, get } from 'svelte/store';
import { getSettings, updateSettings } from '../api/client.js';
import { DEFAULT_SNOOZE_PRESET, type SnoozePreset } from '../utils/snooze.js';

const DENSE_KEY = 'kestrel:settings:dense_mode';
const LANDING_KEY = 'kestrel:settings:landing_view';
const SEND_ACTION_KEY = 'kestrel:settings:send_action';
const SIG_KEY = 'kestrel:settings:signature';
const LABELS_KEY = 'kestrel:settings:label_customizations';

export type MailDensity = 'compact' | 'comfortable' | 'roomy';
const DENSITY_KEY = 'kestrel:settings:density';

/** Resolve the density from stored values, migrating the legacy boolean once. */
export function resolveDensity(stored: string | null, legacy: string | null): MailDensity {
  if (stored === 'compact' || stored === 'comfortable' || stored === 'roomy') return stored;
  if (legacy !== null) return legacy === 'true' ? 'compact' : 'comfortable';
  return 'comfortable';
}

function loadDensity(): MailDensity {
  try {
    return resolveDensity(localStorage.getItem(DENSITY_KEY), localStorage.getItem(DENSE_KEY));
  } catch {
    return 'comfortable';
  }
}

export const mailDensity = writable<MailDensity>(loadDensity());
// Legacy mirror for backend settings sync (compact maps to dense).
export const mailDenseMode = derived(mailDensity, ($d) => $d === 'compact');
// True when the user already picked a 3-way density (snapshot before the
// store subscriber below persists the default on first module load).
const hadLocalDensityChoice = (() => {
  try {
    return localStorage.getItem(DENSITY_KEY) !== null;
  } catch {
    return true;
  }
})();
export const mailDefaultLandingView = writable<string>(loadStr(LANDING_KEY, 'inbox'));
export const mailDefaultSendAction = writable<string>(loadStr(SEND_ACTION_KEY, 'send'));
export const mailSignature = writable<string>(loadStr(SIG_KEY, ''));
export const labelCustomizations = writable<Record<string, { iconName: string; colorName: string }>>(
  loadJson(LABELS_KEY, {})
);
const SYNC_INTERVAL_KEY = 'kestrel:settings:sync_interval';
export const syncInterval = writable<number>(loadNumber(SYNC_INTERVAL_KEY, 300)); // Default 5 mins

export type SwipeActionType = 'archive' | 'trash' | 'toggle_read' | 'toggle_star' | 'snooze' | 'none';
const SWIPE_LEFT_KEY = 'kestrel:settings:swipe_left';
const SWIPE_RIGHT_KEY = 'kestrel:settings:swipe_right';
export const swipeLeftAction = writable<SwipeActionType>(loadStr(SWIPE_LEFT_KEY, 'archive') as SwipeActionType);
export const swipeRightAction = writable<SwipeActionType>(loadStr(SWIPE_RIGHT_KEY, 'toggle_read') as SwipeActionType);
const SNOOZE_DEFAULT_KEY = 'kestrel:settings:snooze_default';
export const mailSnoozeDefault = writable<SnoozePreset>(
  loadStr(SNOOZE_DEFAULT_KEY, DEFAULT_SNOOZE_PRESET) as SnoozePreset
);

let isInitializing = false;
let isUpdating = false;

export async function initializeSettings() {
  if (isInitializing) return;
  isInitializing = true;
  try {
    const settings = await getSettings();
    if (settings.mailDenseMode != null && !hadLocalDensityChoice) {
      mailDensity.set(settings.mailDenseMode ? 'compact' : 'comfortable');
    }
    if (settings.mailDefaultLandingView != null) mailDefaultLandingView.set(settings.mailDefaultLandingView);
    if (settings.mailDefaultSendAction != null) mailDefaultSendAction.set(settings.mailDefaultSendAction);
    if (settings.mailSignature != null) mailSignature.set(settings.mailSignature);
    if (settings.labelCustomizations != null) labelCustomizations.set(settings.labelCustomizations);
    if (settings.syncInterval != null) syncInterval.set(settings.syncInterval);
    if (settings.swipeLeftAction != null) swipeLeftAction.set(settings.swipeLeftAction as SwipeActionType);
    if (settings.swipeRightAction != null) swipeRightAction.set(settings.swipeRightAction as SwipeActionType);
    if (settings.mailSnoozeDefault != null) mailSnoozeDefault.set(settings.mailSnoozeDefault as SnoozePreset);

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
  try {
    // Only sync settings to backend if authenticated
    const authModule = await import('./auth.svelte.js');
    if (!authModule?.authState?.isAuthenticated) return;

    isUpdating = true;
    await updateSettings({
      mailDenseMode: get(mailDenseMode),
      mailDefaultLandingView: get(mailDefaultLandingView),
      mailDefaultSendAction: get(mailDefaultSendAction),
      mailSignature: get(mailSignature),
      labelCustomizations: get(labelCustomizations),
      syncInterval: get(syncInterval),
      swipeLeftAction: get(swipeLeftAction),
      swipeRightAction: get(swipeRightAction),
      mailSnoozeDefault: get(mailSnoozeDefault),
      // we'll update theme too if available
      theme: (typeof localStorage !== 'undefined' ? localStorage.getItem('kestrel:settings:theme') : null) || 'system',
    });
  } catch (err) {
    console.error('Failed to sync settings to backend', err);
  } finally {
    isUpdating = false;
  }
}

// Subscribe & persist settings changes
mailDensity.subscribe((val) => {
  saveItem(DENSITY_KEY, val);
  saveItem(DENSE_KEY, String(val === 'compact'));
  syncToBackend();
});
mailDefaultLandingView.subscribe((val) => {
  saveItem(LANDING_KEY, val);
  syncToBackend();
});
mailDefaultSendAction.subscribe((val) => {
  saveItem(SEND_ACTION_KEY, val);
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
mailSnoozeDefault.subscribe((val) => {
  saveItem(SNOOZE_DEFAULT_KEY, val);
  syncToBackend();
});
swipeLeftAction.subscribe((val) => {
  saveItem(SWIPE_LEFT_KEY, val);
  syncToBackend();
});
swipeRightAction.subscribe((val) => {
  saveItem(SWIPE_RIGHT_KEY, val);
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
