<script lang="ts">
  import { login, authState } from '../stores/auth.svelte.js';

  import { onMount } from 'svelte';

  let username = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state('');

  onMount(() => {
    if (typeof window !== 'undefined') {
      import('@tauri-apps/plugin-deep-link').then(({ onOpenUrl }) => {
        onOpenUrl(async (urls) => {
          for (const url of urls) {
            if (url.startsWith('kestrel://')) {
              try {
                const urlObj = new URL(url);
                const token = urlObj.searchParams.get('token');
                const userId = urlObj.searchParams.get('user_id');
                if (token && userId) {
                  localStorage.setItem('kestrel_token', token);
                  localStorage.setItem('kestrel_user_id', userId);
                  authState.userId = userId;
                }
              } catch(e) {
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

  async function handleLogin(e: Event) {
    e.preventDefault();
    loading = true;
    error = '';
    const res = await login(username, password);
    if (!res.success) {
      error = res.error || 'Login failed';
    }
    loading = false;
  }
</script>

<div class="flex items-center justify-center h-screen w-screen bg-[var(--color-canvas-base)] text-white">
  <div class="w-full max-w-sm p-8 bg-[#1a1a1a] rounded-xl border border-[var(--color-border-hairline)] shadow-2xl">
    <div class="text-center mb-8">
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
        <label for="username" class="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-md focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-colors"
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
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-md focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-colors"
          placeholder="••••••••"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-500 text-white font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed mt-4"
      >
        {loading ? 'Signing in...' : 'Sign In'}
      </button>
    </form>
  </div>
</div>
