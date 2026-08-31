<script lang="ts">
  import { login, ServerConfig } from '@kestrel/shared';

  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let errorMsg = $state('');
  let providers = $state<any[]>([]);
  let providersLoaded = $state(false);

  // Fetch available providers (public endpoint) to render "connect" buttons
  import('@kestrel/shared/api').then(async ({ getProviders }) => {
    try {
      providers = await getProviders();
    } catch (e) {
      console.error('Failed to load providers:', e);
    } finally {
      providersLoaded = true;
    }
  });

  async function handleLogin() {
    loading = true;
    errorMsg = '';

    try {
      await login(email, password);
      window.location.href = '/';
    } catch (e: any) {
      errorMsg = e?.message || 'Login failed. Please check your credentials or host URL.';
    } finally {
      loading = false;
    }
  }

  function connectProvider(providerId: string) {
    import('@kestrel/shared/api').then(({ loginWithProvider }) => {
      loginWithProvider(providerId);
    });
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] p-6">
  <div class="w-full max-w-md bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl p-8 shadow-xl">
    <div class="mb-6 text-center">
      <h1 class="text-2xl font-bold mb-2">Sign in to Kestrel</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">Enter your credentials to access your mail.</p>
    </div>

    {#if errorMsg}
      <div class="mb-4 p-3 bg-red-500/10 border border-red-500/20 text-red-400 text-xs rounded-lg text-center">
        {errorMsg}
      </div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleLogin(); }} class="space-y-4">
      <!-- Host URL Selection directly on the Login screen -->
      <ServerConfig class="pb-2 border-b border-[var(--color-border-hairline)]" />

      <div>
        <label for="email" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-2">
          Email / Username
        </label>
        <input
          id="email"
          type="text"
          bind:value={email}
          placeholder="username@kestrel.dev"
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-white text-sm"
        />
      </div>

      <div>
        <label for="password" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-2">
          Password
        </label>
        <input
          id="password"
          type="password"
          bind:value={password}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg focus:outline-none focus:border-white text-sm"
        />
      </div>

      <button type="submit" disabled={loading} class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-sm rounded-lg transition-colors disabled:opacity-50 cursor-pointer">
        {loading ? 'Signing in...' : 'Sign In'}
      </button>

      <div class="mt-4 flex items-center justify-between text-xs text-[var(--color-text-secondary)]">
        <span>Don't have an account?</span>
        <a href="/register" class="text-blue-400 hover:underline font-semibold">
          Register now &rarr;
        </a>
      </div>
    </form>

    {#if providersLoaded && providers.length > 0}
      <div class="my-6 flex items-center gap-3">
        <div class="flex-1 border-t border-[var(--color-border-hairline)]"></div>
        <span class="text-[10px] uppercase tracking-wider text-[var(--color-text-secondary)]">or connect a provider</span>
        <div class="flex-1 border-t border-[var(--color-border-hairline)]"></div>
      </div>

      <div class="space-y-3">
        {#each providers as provider}
          <button
            onclick={() => connectProvider(provider.id)}
            class="w-full flex items-center justify-center gap-3 p-3 rounded-lg border border-[var(--color-border-hairline)] bg-[var(--color-canvas-base)] hover:bg-[var(--color-canvas-hover)] transition-colors text-sm text-white font-medium cursor-pointer"
          >
            <span style="color: {provider.button_color}" class="w-5 h-5 inline-flex items-center justify-center">
              {@html provider.icon_svg}
            </span>
            {provider.button_text}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
