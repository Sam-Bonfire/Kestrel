<script lang="ts">
  import { onMount } from 'svelte';
  import { getServerUrl, setServerUrl, checkServerHealth } from '../api/client.js';
  import { Server, CheckCircle, AlertCircle, RefreshCw } from 'lucide-svelte';

  interface Props {
    class?: string;
    onUrlChange?: (url: string) => void;
  }

  let { class: className = '', onUrlChange }: Props = $props();

  let serverInputUrl = $state(getServerUrl());
  let testingConnection = $state(false);
  let connectionStatus = $state<{ tested: boolean; ok: boolean; message: string }>({
    tested: false,
    ok: false,
    message: '',
  });

  onMount(() => {
    serverInputUrl = getServerUrl();
    // Pre-test health check quietly on mount
    testServer(serverInputUrl);
  });

  export async function testServer(urlToTest: string) {
    if (!urlToTest.trim()) return;
    testingConnection = true;
    try {
      const normalized = setServerUrl(urlToTest);
      serverInputUrl = normalized;
      if (onUrlChange) onUrlChange(normalized);
      const res = await checkServerHealth(normalized);
      if (res.ok) {
        connectionStatus = { tested: true, ok: true, message: 'Server online' };
      } else {
        connectionStatus = { tested: true, ok: false, message: res.error || 'Cannot reach server' };
      }
    } finally {
      testingConnection = false;
    }
  }

  function handleInputBlur() {
    if (serverInputUrl.trim()) {
      const normalized = setServerUrl(serverInputUrl);
      serverInputUrl = normalized;
      if (onUrlChange) onUrlChange(normalized);
    }
  }
</script>

<div class="space-y-1.5 {className}">
  <div class="flex items-center justify-between">
    <label for="server-host-url" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)]">
      Host / Server URL
    </label>
    {#if connectionStatus.tested}
      <div class="flex items-center gap-1 text-[11px] {connectionStatus.ok ? 'text-green-400' : 'text-red-400'} font-medium">
        {#if connectionStatus.ok}
          <span class="inline-block w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"></span>
          <span>Online</span>
        {:else}
          <span class="inline-block w-1.5 h-1.5 rounded-full bg-red-400"></span>
          <span>Offline</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-2">
    <div class="relative flex-1">
      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-[var(--color-text-secondary)]">
        <Server class="w-3.5 h-3.5" />
      </div>
      <input
        id="server-host-url"
        type="url"
        bind:value={serverInputUrl}
        onblur={handleInputBlur}
        class="w-full pl-9 pr-3 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg text-sm text-white focus:outline-none focus:border-blue-500 transition-colors"
        placeholder="http://localhost:8080"
        required
      />
    </div>

    <button
      type="button"
      onclick={() => testServer(serverInputUrl)}
      disabled={testingConnection}
      class="px-3 py-2 border border-[var(--color-border-hairline)] hover:bg-white/5 text-[var(--color-text-secondary)] hover:text-white rounded-lg transition-colors flex items-center gap-1.5 text-xs cursor-pointer shrink-0 disabled:opacity-50 font-medium"
      title="Test connection to server"
    >
      {#if testingConnection}
        <RefreshCw class="w-3 h-3 animate-spin text-blue-400" />
        <span>Testing</span>
      {:else}
        <RefreshCw class="w-3 h-3" />
        <span>Test</span>
      {/if}
    </button>
  </div>

  {#if connectionStatus.tested && !connectionStatus.ok}
    <p class="text-[11px] text-red-400 flex items-center gap-1 pt-0.5">
      <AlertCircle class="w-3 h-3 shrink-0" />
      <span>{connectionStatus.message}</span>
    </p>
  {/if}
</div>
