<script lang="ts">
  import { Button, Spinner, ErrorBanner } from '@kestrel/shared';
  import { getHealth } from '@kestrel/shared';

  let serverUrl = $state('http://127.0.0.1:8080');
  let loading = $state(false);
  let errorMsg = $state('');
  let success = $state(false);

  async function handleConnect() {
    loading = true;
    errorMsg = '';
    success = false;

    try {
      // Test server connectivity via /health check
      await getHealth();
      success = true;
      localStorage.setItem('kestrel_server_url', serverUrl);
    } catch (e: any) {
      errorMsg = e?.message || 'Failed to connect to Kestrel backend server.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] p-6">
  <div class="w-full max-w-md bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl p-8 shadow-xl">
    <div class="mb-6 text-center">
      <h1 class="text-2xl font-bold mb-2">Setup Kestrel Mail</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">Enter your Kestrel backend server URL to connect.</p>
    </div>

    {#if errorMsg}
      <div class="mb-4">
        <ErrorBanner message={errorMsg} />
      </div>
    {/if}

    {#if success}
      <div class="mb-4 p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-lg text-emerald-400 text-sm text-center">
        Connected successfully! Redirecting to login...
      </div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleConnect(); }} class="space-y-4">
      <div>
        <label for="serverUrl" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-2">
          Backend Server URL
        </label>
        <input
          id="serverUrl"
          type="text"
          bind:value={serverUrl}
          placeholder="http://127.0.0.1:8080"
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-white text-sm"
        />
      </div>

      <Button type="submit" variant="primary" size="md" disabled={loading} className="w-full justify-center">
        {#if loading}
          <Spinner size="sm" />
          <span class="ml-2">Connecting...</span>
        {:else}
          Connect to Server
        {/if}
      </Button>
    </form>
  </div>
</div>
