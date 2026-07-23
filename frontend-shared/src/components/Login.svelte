<script lang="ts">
  import { login } from '../stores/auth.svelte.js';

  let username = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state('');

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
