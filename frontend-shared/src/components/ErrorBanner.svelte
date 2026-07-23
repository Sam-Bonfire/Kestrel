<script>
  import { createEventDispatcher } from 'svelte';

  export let message = 'Something went wrong.';

  const dispatch = createEventDispatcher();

  function handleRetry() {
    dispatch('retry');
  }
</script>

<div class="error-banner" role="alert">
  <div class="error-banner__body">
    <svg
      class="error-banner__icon"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    >
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="12" />
      <line x1="12" y1="16" x2="12.01" y2="16" />
    </svg>
    <span class="error-banner__text">{message}</span>
  </div>
  <button class="error-banner__retry" on:click={handleRetry}>
    Retry
  </button>
</div>

<style>
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.625rem 0.875rem;
    background: rgba(127, 29, 29, 0.2);
    border: 1px solid rgba(153, 27, 27, 0.3);
    border-radius: 8px;
    animation: fadeIn 200ms ease-out;
  }
  .error-banner__body {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .error-banner__icon {
    color: #f87171;
    flex-shrink: 0;
  }
  .error-banner__text {
    color: #fca5a5;
    font-size: 0.75rem;
    line-height: 1.4;
  }
  .error-banner__retry {
    flex-shrink: 0;
    padding: 0.25rem 0.625rem;
    background: transparent;
    border: 1px solid rgba(153, 27, 27, 0.4);
    border-radius: 6px;
    color: #f87171;
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 120ms;
  }
  .error-banner__retry:hover {
    background: rgba(127, 29, 29, 0.4);
    color: #fca5a5;
  }
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
