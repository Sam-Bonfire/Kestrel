<script lang="ts">
  import { X, Mail, Calendar, RefreshCw, Trash2, Plus, Server, CheckCircle, AlertCircle } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { apiClient, getServerUrl, setServerUrl, resetServerUrl, checkServerHealth } from '../api/client.js';

  let { isOpen = false, onClose = () => {} } = $props<{
    isOpen?: boolean;
    onClose?: () => void;
  }>();

  let activeTab = $state('shortcuts'); // 'shortcuts' | 'accounts' | 'server'

  // --- Server State ---
  let serverUrl = $state(getServerUrl());
  let serverTesting = $state(false);
  let serverStatus = $state<{ tested: boolean; ok: boolean; message: string }>({
    tested: false,
    ok: false,
    message: '',
  });

  function handleSaveServer() {
    if (!serverUrl.trim()) return;
    const normalized = setServerUrl(serverUrl);
    serverUrl = normalized;
    testServerConnection();
  }

  function handleResetServer() {
    resetServerUrl();
    serverUrl = getServerUrl();
    testServerConnection();
  }

  async function testServerConnection() {
    serverTesting = true;
    serverStatus = { tested: false, ok: false, message: '' };
    try {
      const res = await checkServerHealth(serverUrl);
      if (res.ok) {
        serverStatus = { tested: true, ok: true, message: 'Server online and reachable' };
      } else {
        serverStatus = { tested: true, ok: false, message: res.error || 'Cannot reach server' };
      }
    } finally {
      serverTesting = false;
    }
  }

  // --- Shortcuts State ---
  let shortcuts = $state([
    { id: 'compose', label: 'Compose New Message', key: 'C' },
    { id: 'command', label: 'Command Palette', key: 'Cmd/Ctrl + K' },
    { id: 'day_view', label: 'Day View (Calendar)', key: 'D' },
    { id: 'week_view', label: 'Week View (Calendar)', key: 'W' },
  ]);

  let recordingId: string | null = $state(null);
  let recordingKey = $state('');

  function startRecording(id: string) {
    recordingId = id;
    recordingKey = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recordingId) return;
    
    e.preventDefault();
    e.stopPropagation();

    if (['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) return;

    let keys = [];
    if (e.metaKey) keys.push('Cmd');
    if (e.ctrlKey) keys.push('Ctrl');
    if (e.altKey) keys.push('Alt');
    if (e.shiftKey) keys.push('Shift');
    keys.push(e.key.toUpperCase());

    const combo = keys.join(' + ');
    
    shortcuts = shortcuts.map(s => s.id === recordingId ? { ...s, key: combo } : s);
    recordingId = null;
  }

  // --- Accounts State ---
  let accounts: any[] = $state([]);
  let providers: any[] = $state([]);
  let isLoadingAccounts = $state(false);

  async function loadAccounts() {
    isLoadingAccounts = true;
    try {
      accounts = await apiClient.get('/api/v1/accounts');
      providers = await apiClient.get('/api/v1/providers');
    } catch (e) {
      console.error("Failed to load accounts/providers", e);
    } finally {
      isLoadingAccounts = false;
    }
  }

  $effect(() => {
    if (isOpen && activeTab === 'accounts') {
      loadAccounts();
    }
  });

  async function disconnectAccount(id: string) {
    if (!confirm('Are you sure you want to disconnect this account? This will remove all synced data.')) return;
    try {
      await apiClient.delete(`/api/v1/accounts/${id}`);
      await loadAccounts();
    } catch (e) {
      console.error("Failed to disconnect account", e);
    }
  }

  async function syncAccount(id: string) {
    try {
      await apiClient.post('/api/v1/sync/trigger', { account_id: id });
      alert('Sync triggered successfully.');
    } catch (e) {
      console.error("Failed to sync account", e);
    }
  }

  function connectProvider(providerId: string) {
    // Use the shared helper so the correct backend base URL (and OAuth flow) is used.
    import('@kestrel/shared/api').then(({ loginWithProvider }) => {
      loginWithProvider(providerId);
    });
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs" onkeydown={handleKeyDown} tabindex="-1">
    <div class="w-full max-w-2xl bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl shadow-2xl overflow-hidden font-sans" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-border-hairline)]">
        <h2 class="text-lg font-semibold text-white">Settings</h2>
        <button onclick={onClose} class="p-1 hover:bg-white/10 rounded-md text-[var(--color-text-secondary)] transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>
      
      <div class="p-4 flex h-[400px]">
        <!-- Sidebar -->
        <div class="w-48 border-r border-[var(--color-border-hairline)] pr-4 flex flex-col gap-1">
          <button 
            onclick={() => activeTab = 'shortcuts'}
            class="w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'shortcuts' ? 'bg-[var(--color-canvas-hover)] text-white' : 'text-[var(--color-text-secondary)] hover:bg-white/5'}">
            Keyboard Shortcuts
          </button>
          <button 
            onclick={() => activeTab = 'accounts'}
            class="w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'accounts' ? 'bg-[var(--color-canvas-hover)] text-white' : 'text-[var(--color-text-secondary)] hover:bg-white/5'}">
            Accounts & Sync
          </button>
          <button 
            onclick={() => { activeTab = 'server'; testServerConnection(); }}
            class="w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'server' ? 'bg-[var(--color-canvas-hover)] text-white' : 'text-[var(--color-text-secondary)] hover:bg-white/5'}">
            Server & Network
          </button>
        </div>
        
        <!-- Content -->
        <div class="flex-1 pl-4 overflow-y-auto">
          
          {#if activeTab === 'shortcuts'}
            <h3 class="text-white font-medium mb-4">Custom Shortcuts</h3>
            <p class="text-[var(--color-text-secondary)] text-sm mb-6">Click on a shortcut to record a new key combination.</p>
            
            <div class="space-y-3">
              {#each shortcuts as shortcut}
                <div class="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)]">
                  <span class="text-sm text-white">{shortcut.label}</span>
                  <button
                    onclick={() => startRecording(shortcut.id)}
                    class="px-3 py-1.5 rounded-md text-xs font-mono font-medium border border-white/20 hover:border-white/40 transition-colors {recordingId === shortcut.id ? 'bg-blue-500/20 border-blue-500 text-blue-400' : 'bg-[#1a1919] text-[var(--color-text-secondary)]'}"
                  >
                    {recordingId === shortcut.id ? 'Recording...' : shortcut.key}
                  </button>
                </div>
              {/each}
            </div>
            
          {:else if activeTab === 'accounts'}
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-white font-medium">Connected Accounts</h3>
            </div>
            
            {#if isLoadingAccounts}
              <div class="text-[var(--color-text-secondary)] text-sm py-4">Loading accounts...</div>
            {:else}
              <div class="space-y-3 mb-8">
                {#if accounts.length === 0}
                  <div class="p-4 rounded-lg border border-dashed border-[var(--color-border-hairline)] text-center text-[var(--color-text-secondary)] text-sm">
                    No accounts connected yet.
                  </div>
                {/if}
                
                {#each accounts as account}
                  <div class="flex flex-col p-3 rounded-lg border border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)]">
                    <div class="flex items-center justify-between mb-2">
                      <div class="flex items-center gap-2">
                        <Mail class="w-4 h-4 text-white" />
                        <span class="text-sm text-white font-medium">{account.display_name}</span>
                        <span class="text-xs px-2 py-0.5 rounded-full bg-white/10 text-[var(--color-text-secondary)] uppercase">{account.provider}</span>
                      </div>
                      <div class="flex items-center gap-2">
                        <button onclick={() => syncAccount(account.id)} title="Sync Now" class="p-1.5 hover:bg-white/10 rounded text-[var(--color-text-secondary)] transition-colors">
                          <RefreshCw class="w-3.5 h-3.5" />
                        </button>
                        <button onclick={() => disconnectAccount(account.id)} title="Disconnect" class="p-1.5 hover:bg-red-500/20 text-red-400 rounded transition-colors">
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                    <div class="text-xs text-[var(--color-text-secondary)] flex items-center gap-2">
                      <span>Last synced: {new Date(account.updated_at * 1000).toLocaleString()}</span>
                    </div>
                  </div>
                {/each}
              </div>
              
              <h3 class="text-white font-medium mb-3">Add Account</h3>
              <div class="space-y-3">
                {#each providers as provider}
                  <button 
                    onclick={() => connectProvider(provider.name.toLowerCase())}
                    class="w-full flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)] hover:bg-[var(--color-canvas-hover)] transition-colors text-left"
                  >
                    <div class="flex items-center gap-3">
                      <div class="w-6 h-6 flex items-center justify-center rounded" style="color: {provider.button_color}">
                        {@html provider.icon_svg}
                      </div>
                      <span class="text-sm text-white">{provider.button_text}</span>
                    </div>
                    <Plus class="w-4 h-4 text-[var(--color-text-secondary)]" />
                  </button>
                {/each}
                {#if providers.length === 0}
                  <div class="text-sm text-[var(--color-text-secondary)]">No plugins loaded.</div>
                {/if}
              </div>
            {/if}

          {:else if activeTab === 'server'}
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-white font-medium">Backend Server & Network</h3>
            </div>
            
            <p class="text-[var(--color-text-secondary)] text-sm mb-6">
              Configure the remote Kestrel server or NAS endpoint this client communicates with.
            </p>

            <div class="space-y-4 bg-[var(--color-canvas-base)] p-4 rounded-lg border border-[var(--color-border-hairline)]">
              <div>
                <label for="settings-server-url" class="block text-sm font-medium text-white mb-1.5">
                  Server Endpoint URL
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="settings-server-url"
                    type="url"
                    bind:value={serverUrl}
                    class="flex-1 px-3 py-2 bg-[#121212] border border-[var(--color-border-hairline)] rounded-md text-sm text-white focus:outline-none focus:border-blue-500"
                    placeholder="https://kestrel.yourdomain.com"
                  />
                  <button
                    onclick={handleSaveServer}
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-md transition-colors"
                  >
                    Save
                  </button>
                </div>
              </div>

              <div class="flex items-center justify-between pt-2 border-t border-[var(--color-border-hairline)]">
                <div class="flex items-center gap-2 text-xs">
                  {#if serverStatus.tested}
                    <div class="flex items-center gap-1.5 {serverStatus.ok ? 'text-green-400' : 'text-red-400'} font-medium">
                      {#if serverStatus.ok}
                        <CheckCircle class="w-4 h-4 shrink-0" />
                      {:else}
                        <AlertCircle class="w-4 h-4 shrink-0" />
                      {/if}
                      <span>{serverStatus.message}</span>
                    </div>
                  {:else}
                    <span class="text-[var(--color-text-secondary)]">Click test to verify server connectivity</span>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  <button
                    onclick={testServerConnection}
                    disabled={serverTesting}
                    class="px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white text-xs font-medium rounded-md transition-colors flex items-center gap-1.5"
                  >
                    {#if serverTesting}
                      <RefreshCw class="w-3.5 h-3.5 animate-spin" />
                      Testing...
                    {:else}
                      <RefreshCw class="w-3.5 h-3.5" />
                      Test Connectivity
                    {/if}
                  </button>
                  <button
                    onclick={handleResetServer}
                    class="px-3 py-1.5 border border-[var(--color-border-hairline)] hover:bg-white/5 text-[var(--color-text-secondary)] text-xs font-medium rounded-md transition-colors"
                  >
                    Reset to Default
                  </button>
                </div>
              </div>
            </div>
          {/if}
          
        </div>
      </div>
    </div>
  </div>
{/if}
