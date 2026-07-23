<script lang="ts">
  import { Button, Spinner, ErrorBanner, login } from '@kestrel/shared';

  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let errorMsg = $state('');

  async function handleLogin() {
    loading = true;
    errorMsg = '';

    try {
      await login(email, password);
      window.location.href = '/';
    } catch (e: any) {
      errorMsg = e?.message || 'Login failed. Please check your credentials.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] p-6">
  <div class="w-full max-w-md bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl p-8 shadow-xl">
    <div class="mb-6 text-center">
      <h1 class="text-2xl font-bold mb-2">Sign in to Kestrel</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">Enter your credentials to access your mail.</p>
    </div>

    {#if errorMsg}
      <div class="mb-4">
        <ErrorBanner message={errorMsg} />
      </div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleLogin(); }} class="space-y-4">
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

      <Button type="submit" variant="primary" size="md" disabled={loading} className="w-full justify-center">
        {#if loading}
          <Spinner size="sm" />
          <span class="ml-2">Signing in...</span>
        {:else}
          Sign In
        {/if}
      </Button>

      <div class="mt-4 text-center text-xs text-[var(--color-text-secondary)]">
        Don't have an account? <a href="/register" class="text-white hover:underline">Register</a>
      </div>
    </form>
  </div>
</div>
