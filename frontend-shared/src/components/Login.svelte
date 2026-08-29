<script lang="ts">
  import { login, authState } from '../stores/auth.svelte.js';
  import { register, setServerUrl } from '../api/client.js';
  import { onMount } from 'svelte';
  import ServerConfig from './ServerConfig.svelte';

  interface Props {
    initialMode?: 'login' | 'register';
    onSuccess?: () => void;
  }

  let { initialMode = 'login', onSuccess }: Props = $props();

  let mode = $state<'login' | 'register'>(initialMode);
  let username = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let loading = $state(false);
  let error = $state('');
  let successMsg = $state('');

  onMount(() => {
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
                  setServerUrl(serverParam);
                }

                // 2. Check for token/auth deep link
                const token = urlObj.searchParams.get('token');
                const userId = urlObj.searchParams.get('user_id');
                if (token && userId) {
                  localStorage.setItem('kestrel_token', token);
                  localStorage.setItem('kestrel_user_id', userId);
                  authState.userId = userId;
                  if (onSuccess) onSuccess();
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

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = '';
    successMsg = '';

    if (mode === 'register') {
      if (password.length < 8) {
        error = 'Password must be at least 8 characters long.';
        loading = false;
        return;
      }
      if (password !== confirmPassword) {
        error = 'Passwords do not match.';
        loading = false;
        return;
      }

      try {
        await register(username, password);
        const res = await login(username, password);
        if (res.success) {
          if (onSuccess) onSuccess();
        } else {
          error = res.error || 'Account created, but sign in failed. Please sign in manually.';
          mode = 'login';
        }
      } catch (err: any) {
        error = err?.message || 'Registration failed. Username may already exist.';
      } finally {
        loading = false;
      }
    } else {
      const res = await login(username, password);
      if (!res.success) {
        error = res.error || 'Login failed. Please check credentials or host URL.';
      } else {
        if (onSuccess) onSuccess();
      }
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen w-screen bg-[var(--color-canvas-base)] text-white p-4">
  <div class="w-full max-w-md p-8 bg-[#1a1a1a] rounded-xl border border-[var(--color-border-hairline)] shadow-2xl">
    <div class="text-center mb-6">
      <h1 class="text-2xl font-semibold mb-2">Kestrel</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">
        {mode === 'login' ? 'Sign in to your workspace' : 'Create your Kestrel account'}
      </p>
    </div>

    <!-- Mode Selector Tabs -->
    <div class="flex rounded-lg bg-[var(--color-canvas-base)] p-1 border border-[var(--color-border-hairline)] mb-6 text-xs font-medium">
      <button
        type="button"
        onclick={() => { mode = 'login'; error = ''; }}
        class="flex-1 py-1.5 rounded-md transition-all cursor-pointer {mode === 'login' ? 'bg-blue-600 text-white shadow' : 'text-[var(--color-text-secondary)] hover:text-white'}"
      >
        Sign In
      </button>
      <button
        type="button"
        onclick={() => { mode = 'register'; error = ''; }}
        class="flex-1 py-1.5 rounded-md transition-all cursor-pointer {mode === 'register' ? 'bg-blue-600 text-white shadow' : 'text-[var(--color-text-secondary)] hover:text-white'}"
      >
        Register Now
      </button>
    </div>

    {#if error}
      <div class="p-3 mb-6 bg-red-500/10 border border-red-500/20 text-red-400 text-xs rounded-lg text-center">
        {error}
      </div>
    {/if}

    {#if successMsg}
      <div class="p-3 mb-6 bg-green-500/10 border border-green-500/20 text-green-400 text-xs rounded-lg text-center">
        {successMsg}
      </div>
    {/if}

    <form onsubmit={handleSubmit} class="space-y-4">
      <!-- Host URL Selection right on the login form -->
      <ServerConfig class="pb-2" />

      <div>
        <label for="username" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-1">
          Username / Email
        </label>
        <input
          id="username"
          type="text"
          bind:value={username}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-blue-500 transition-colors text-sm text-white"
          placeholder="username@kestrel.dev"
        />
      </div>

      <div>
        <label for="password" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-1">
          Password
        </label>
        <input
          id="password"
          type="password"
          bind:value={password}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-blue-500 transition-colors text-sm text-white"
          placeholder="••••••••"
        />
      </div>

      {#if mode === 'register'}
        <div>
          <label for="confirm-password" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-1">
            Confirm Password
          </label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            required
            class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-blue-500 transition-colors text-sm text-white"
            placeholder="••••••••"
          />
        </div>
      {/if}

      <button
        type="submit"
        disabled={loading}
        class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-500 text-white font-medium text-sm rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed mt-2 cursor-pointer"
      >
        {#if loading}
          {mode === 'login' ? 'Signing in...' : 'Creating account...'}
        {:else}
          {mode === 'login' ? 'Sign In' : 'Register Now'}
        {/if}
      </button>

      <div class="pt-2 text-center text-xs text-[var(--color-text-secondary)]">
        {#if mode === 'login'}
          Don't have an account?
          <button
            type="button"
            onclick={() => { mode = 'register'; error = ''; }}
            class="text-blue-400 hover:underline font-medium ml-1 cursor-pointer"
          >
            Register now
          </button>
        {:else}
          Already have an account?
          <button
            type="button"
            onclick={() => { mode = 'login'; error = ''; }}
            class="text-blue-400 hover:underline font-medium ml-1 cursor-pointer"
          >
            Sign in
          </button>
        {/if}
      </div>
    </form>
  </div>
</div>
