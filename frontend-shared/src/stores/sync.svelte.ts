import { API_BASE } from '../api/client';
import { authState } from './auth.svelte';

export interface SyncState {
  isSyncing: boolean;
  message: string;
  progress: number; // 0-100
  lastSyncTime: number | null;
  error: string | null;
}

const syncStates = $state<Record<string, SyncState>>({});
let globalIsSyncing = $state(false);
let globalSyncMessage = $state('');

export function getSyncState(accountId: string): SyncState {
  if (!syncStates[accountId]) {
    syncStates[accountId] = {
      isSyncing: false,
      message: '',
      progress: 0,
      lastSyncTime: null,
      error: null,
    };
  }
  return syncStates[accountId];
}

export function getGlobalSyncState() {
  return {
    get isSyncing() {
      return globalIsSyncing;
    },
    get message() {
      return globalSyncMessage;
    }
  };
}

let eventSource: EventSource | null = null;

export function initSyncEvents() {
  if (eventSource) return;

  const token = authState.token;
  if (!token) return;

  eventSource = new EventSource(`${API_BASE}/sync/stream?token=${token}`);

  eventSource.addEventListener('sync_started', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);
      const state = getSyncState(data.account_id);
      state.isSyncing = true;
      state.message = `Syncing ${data.provider}...`;
      state.error = null;
      state.progress = 0;

      globalIsSyncing = true;
      globalSyncMessage = state.message;
    } catch (e) {
      console.error('Failed to parse sync_started event', e);
    }
  });

  eventSource.addEventListener('sync_progress', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);
      const state = getSyncState(data.account_id);

      let progressText = data.stage;
      if (data.total_items) {
        progressText += ` (${data.items_synced}/${data.total_items})`;
        state.progress = Math.floor((data.items_synced / data.total_items) * 100);
      } else {
        progressText += ` (${data.items_synced} items)`;
      }

      state.message = `Syncing ${progressText}...`;

      globalIsSyncing = true;
      globalSyncMessage = state.message;
    } catch (e) {
      console.error('Failed to parse sync_progress event', e);
    }
  });

  eventSource.addEventListener('sync_complete', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);
      const state = getSyncState(data.account_id);
      state.isSyncing = false;
      state.lastSyncTime = Date.now();
      state.progress = 100;
      state.message = 'Sync complete';

      // Update global state if no other accounts are syncing
      updateGlobalSyncState();
    } catch (e) {
      console.error('Failed to parse sync_complete event', e);
    }
  });

  eventSource.addEventListener('sync_error', (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);
      const state = getSyncState(data.account_id);
      state.isSyncing = false;
      state.error = data.error;

      updateGlobalSyncState();
    } catch (e) {
      console.error('Failed to parse sync_error event', e);
    }
  });

  eventSource.onerror = (err) => {
    console.error('SSE connection error:', err);
    // Optionally close and attempt reconnect if necessary
    // eventSource?.close();
    // setTimeout(initSyncEvents, 5000);
  };
}

function updateGlobalSyncState() {
  const syncingAccounts = Object.values(syncStates).filter(s => s.isSyncing);
  if (syncingAccounts.length > 0) {
    globalIsSyncing = true;
    globalSyncMessage = syncingAccounts[0].message; // just show the first one
  } else {
    globalIsSyncing = false;
    globalSyncMessage = '';
  }
}

export function closeSyncEvents() {
  if (eventSource) {
    eventSource.close();
    eventSource = null;
  }
}
