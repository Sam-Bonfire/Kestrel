<script lang="ts">
  import { Button, Spinner, ErrorBanner, register, login, ServerConfig } from '@kestrel/shared';

  let email = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let loading = $state(false);
  let errorMsg = $state('');

  async function handleRegister() {
    if (password.length < 8) {
      errorMsg = 'Password must be at least 8 characters long.';
      return;
    }
    if (password !== confirmPassword) {
      errorMsg = 'Passwords do not match.';
      return;
    }

    loading = true;
    errorMsg = '';

    try {
      await register(email, password);
      // Auto-login after successful registration
      await login(email, password);
      window.location.href = '/';
    } catch (e: any) {
      errorMsg = e?.message || 'Registration failed. Please try again.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-canvas-base)] text-[var(--color-text-primary)] p-6">
  <div class="w-full max-w-md bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-xl p-8 shadow-xl">
    <div class="mb-6 text-center">
      <h1 class="text-2xl font-bold mb-2">Create a Kestrel Account</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">Sign up to access your calendar and schedules.</p>
    </div>

    {#if errorMsg}
      <div class="mb-4">
        <ErrorBanner message={errorMsg} />
      </div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleRegister(); }} class="space-y-4">
      <!-- Host URL Selection directly on the Register screen -->
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
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg text-sm text-white focus:outline-none focus:border-white"
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
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg text-sm text-white focus:outline-none focus:border-white"
        />
      </div>

      <div>
        <label for="confirmPassword" class="block text-xs font-semibold uppercase tracking-wider text-[var(--color-text-secondary)] mb-2">
          Confirm Password
        </label>
        <input
          id="confirmPassword"
          type="password"
          bind:value={confirmPassword}
          required
          class="w-full px-4 py-2 bg-[var(--color-canvas-base)] border border-[var(--color-border-hairline)] rounded-lg text-sm text-white focus:outline-none focus:border-white"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-sm rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
      >
        {loading ? 'Creating account...' : 'Register Now'}
      </button>

      <div class="mt-4 flex items-center justify-between text-xs text-[var(--color-text-secondary)]">
        <span>Already have an account?</span>
        <a href="/login" class="text-blue-400 hover:underline font-semibold">
          &larr; Sign In
        </a>
      </div>
    </form>
  </div>
</div>
