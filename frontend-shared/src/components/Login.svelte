<script lang="ts">
  import { login, authState } from '../stores/auth.svelte.js';
  import { getServerUrl, setServerUrl, checkServerHealth } from '../api/client.js';
  import { onMount } from 'svelte';
  import { Server, CheckCircle, AlertCircle, RefreshCw, ChevronDown, ChevronUp } from 'lucide-svelte';

  let username = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state('');

  // Server configuration state
  let showServerConfig = $state(false);
  let currentServerUrl = $state(getServerUrl());
  let serverInputUrl = $state(getServerUrl());
  let testingConnection = $state(false);
  let connectionStatus = $state<{ tested: boolean; ok: boolean; message: string }>({
    tested: false,
    ok: false,
    message: '',
  });

  onMount(() => {
    currentServerUrl = getServerUrl();
    serverInputUrl = currentServerUrl;

    if (typeof window !== 'undefined') {
      import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
        onOpenUrl(async (urls) => {
          for (const url of urls) {
            if (url.startsWith('kestrel://')) {
              try {
                const urlObj = new URL(url);
                
                // 1. Check for server configuration deep link
                const serverParam = urlObj.searchParams.get('server');
                if (serverParam) {
                  const normalized = setServerUrl(serverParam);
                  currentServerUrl = normalized;
                  serverInputUrl = normalized;
                  await testServer(normalized);
                }

                // 2. Check for token/auth deep link
                const token = urlObj.searchParams.get('token');
                const userId = urlObj.searchParams.get('user_id');
                if (token && userId) {
                  localStorage.setItem('kestrel_token', token);
                  localStorage.setItem('kestrel_user_id', userId);
                  authState.userId = userId;
                }
              } catch (e) {
                console.error("Failed to parse deep link", e);
              }
            }
          }
        }).catch(err => {
          console.error("Failed to register deep link handler", err);
        });
      }).catch(err => {
        console.log('Deep link plugin not available', err);
      });
    }
  });

  async function testServer(urlToTest: string) {
    testingConnection = true;
    connectionStatus = { tested: false, ok: false, message: '' };
    try {
      const res = await checkServerHealth(urlToTest);
      if (res.ok) {
        connectionStatus = { tested: true, ok: true, message: 'Connected to Kestrel server' };
      } else {
        connectionStatus = { tested: true, ok: false, message: res.error || 'Connection failed' };
      }
    } finally {
      testingConnection = false;
    }
  }

  function handleSaveServer() {
    if (!serverInputUrl.trim()) return;
    const normalized = setServerUrl(serverInputUrl);
    currentServerUrl = normalized;
    serverInputUrl = normalized;
    testServer(normalized);
  }

  async function handleLogin(e: Event) {
    e.preventDefault();
    loading = true;
    error = '';
    const res = await login(username, password);
    if (!res.success) {
      error = res.error || 'Login failed. Please check credentials or server URL.';
    }
    loading = false;
  }
</script>

<div class="flex items-center justify-center min-h-screen w-screen bg-[var(--color-canvas-base)] text-white p-4">
  <div class="w-full max-w-sm p-8 bg-[#1a1a1a] rounded-xl border border-[var(--color-border-hairline)] shadow-2xl">
    <div class="text-center mb-6">
      <h1 class="text-2xl font-semibold mb-2">Kestrel</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">Sign in to your workspace</p>
    </div>

    {#if error}
      <div class="p-3 mb-6 bg-red-500/10 border border-red-500/20 text-red-400 text-sm rounded-md text-center">
        {error}
      </div>
    {/if}

    <form onsubmit={handleLogin} class="space-y-4">
      <div>
        <label for="username" class="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">Username / Email</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-md focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-colors text-sm"
          placeholder="demo_user"
        />
      </div>

      <div>
        <label for="password" class="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-md focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-colors text-sm"
          placeholder="••••••••"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-500 text-white font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed mt-2"
      >
        {loading ? 'Signing in...' : 'Sign In'}
      </button>
    </form>

    <!-- Server Configuration Expander -->
    <div class="mt-6 pt-4 border-t border-[var(--color-border-hairline)]">
      <button
        type="button"
        onclick={() => showServerConfig = !showServerConfig}
        class="w-full flex items-center justify-between text-xs text-[var(--color-text-secondary)] hover:text-white transition-colors"
      >
        <div class="flex items-center gap-1.5 truncate">
          <Server class="w-3.5 h-3.5 shrink-0" />
          <span class="truncate">Server: {currentServerUrl}</span>
        </div>
        {#if showServerConfig}
          <ChevronUp class="w-3.5 h-3.5 shrink-0 ml-1" />
        {:else}
          <ChevronDown class="w-3.5 h-3.5 shrink-0 ml-1" />
        {/if}
      </button>

      {#if showServerConfig}
        <div class="mt-3 space-y-2 bg-[var(--color-canvas-base)] p-3 rounded-md border border-[var(--color-border-hairline)] text-xs">
          <label for="server-url" class="block text-[var(--color-text-secondary)] font-medium">Custom Server URL</label>
          <input
            id="server-url"
            type="url"
            bind:value={serverInputUrl}
            class="w-full px-2.5 py-1.5 bg-[#121212] border border-[var(--color-border-hairline)] rounded focus:outline-none focus:border-blue-500 text-xs"
            placeholder="https://kestrel.yourdomain.com"
          />

          <div class="flex items-center gap-2 pt-1">
            <button
              type="button"
              onclick={handleSaveServer}
              class="px-3 py-1 bg-white/10 hover:bg-white/20 text-white rounded transition-colors font-medium"
            >
              Save URL
            </button>
            <button
              type="button"
              onclick={() => testServer(serverInputUrl)}
              disabled={testingConnection}
              class="px-3 py-1 border border-[var(--color-border-hairline)] hover:bg-white/5 text-[var(--color-text-secondary)] rounded transition-colors flex items-center gap-1"
            >
              {#if testingConnection}
                <RefreshCw class="w-3 h-3 animate-spin" />
                Testing...
              {:else}
                Test Connection
              {/if}
            </button>
          </div>

          {#if connectionStatus.tested}
            <div class="flex items-center gap-1.5 pt-1 text-xs {connectionStatus.ok ? 'text-green-400' : 'text-red-400'}">
              {#if connectionStatus.ok}
                <CheckCircle class="w-3.5 h-3.5 shrink-0" />
              {:else}
                <AlertCircle class="w-3.5 h-3.5 shrink-0" />
              {/if}
              <span>{connectionStatus.message}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
