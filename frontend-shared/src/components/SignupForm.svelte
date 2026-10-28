<script lang="ts">
  import { onMount } from 'svelte';
  import { createFormsClient } from '../forms/index.js';
  import { buildSignupFields, MAX_ATTACHMENT_BYTES } from '../forms/fields.js';
  import { loadTurnstile, type TurnstileApi } from '../forms/turnstile.js';

  interface SignupFormProps {
    app: 'landing' | 'mail' | 'calendar';
    form: string;
    endpoint: string;
    token?: string;
    variant?: 'email' | 'application';
    turnstileSiteKey?: string;
    heading?: string;
    blurb?: string;
    submitLabel?: string;
    accept?: string;
  }

  let {
    app,
    form,
    endpoint,
    token = '',
    variant = 'email',
    turnstileSiteKey = '',
    heading = '',
    blurb = '',
    submitLabel = 'Request access',
    accept = '.pdf,.png,.jpg,.jpeg',
  }: SignupFormProps = $props();

  // An unconfigured deployment renders nothing at all: no empty box, no request.
  let enabled = $derived(endpoint.trim() !== '');

  let email = $state('');
  let name = $state('');
  let message = $state('');
  let attachment = $state<File | null>(null);
  let honeypot = $state('');
  let turnstileToken = $state('');
  let status = $state<'idle' | 'sending' | 'sent' | 'error'>('idle');
  let error = $state('');

  let formEl: HTMLFormElement | undefined = $state();
  let turnstileEl: HTMLDivElement | undefined = $state();

  onMount(() => {
    const container = turnstileEl;
    if (!turnstileSiteKey || !container) return;

    let api: TurnstileApi | null = null;
    let widgetId = '';

    loadTurnstile()
      .then((loaded) => {
        api = loaded;
        widgetId = loaded.render(container, {
          sitekey: turnstileSiteKey,
          callback: (value) => {
            turnstileToken = value;
          },
        });
      })
      .catch((cause: unknown) => {
        status = 'error';
        error = cause instanceof Error ? cause.message : 'Security check failed to load';
      });

    return () => {
      if (api && widgetId) api.remove(widgetId);
    };
  });

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (status === 'sending') return;
    if (formEl && !formEl.checkValidity()) {
      formEl.reportValidity?.();
      return;
    }

    status = 'sending';
    error = '';

    try {
      const fields = await buildSignupFields({ email, name, message, attachment });
      fields._hp = honeypot;

      const result = await createFormsClient({ endpoint, token }).submit({
        app,
        form,
        fields,
        meta: {
          // shortcut: Turnstile tokens expire after ~5 minutes; a stale form
          // needs a reload, the receiver rejects the expired token.
          turnstile: turnstileToken || undefined,
          userAgent: navigator.userAgent,
          locale: navigator.language,
        },
      });

      if (result.status === 'sent') {
        status = 'sent';
        return;
      }
      status = 'error';
      error =
        result.status === 'disabled'
          ? 'This form is not configured on this deployment.'
          : result.error;
    } catch (cause) {
      status = 'error';
      error = cause instanceof Error ? cause.message : 'Could not send the form';
    }
  }
</script>

{#if enabled}
  <form bind:this={formEl} onsubmit={submit} class="signup">
    {#if heading}<h3>{heading}</h3>{/if}
    {#if blurb}<p class="blurb">{blurb}</p>{/if}

    {#if status === 'sent'}
      <p class="ok" role="status">Thanks &mdash; your message is on its way.</p>
    {:else}
      <div class="row">
        <label>
          <span>Email</span>
          <input
            type="email"
            name="email"
            required
            autocomplete="email"
            bind:value={email}
            disabled={status === 'sending'}
          />
        </label>

        {#if variant === 'application'}
          <label>
            <span>Name</span>
            <input type="text" name="name" autocomplete="name" bind:value={name} disabled={status === 'sending'} />
          </label>
        {/if}
      </div>

      {#if variant === 'application'}
        <label class="full">
          <span>What are you building, and with which provider?</span>
          <textarea name="message" rows="4" bind:value={message} disabled={status === 'sending'}></textarea>
        </label>

        <label class="full">
          <span>Attachment (optional, max 2 MB)</span>
          <input
            type="file"
            name="attachment"
            {accept}
            onchange={(event) => (attachment = event.currentTarget.files?.[0] ?? null)}
            disabled={status === 'sending'}
          />
        </label>
      {/if}

      <!-- Honeypot: hidden from people, filled by naive bots. -->
      <div class="hp" aria-hidden="true">
        <label>
          <span>Company website</span>
          <input type="text" name="_hp" tabindex="-1" autocomplete="off" bind:value={honeypot} />
        </label>
      </div>

      {#if turnstileSiteKey}
        <div class="turnstile" bind:this={turnstileEl}></div>
      {/if}

      {#if status === 'error'}
        <p class="err" role="alert">{error}</p>
      {/if}

      <button type="submit" disabled={status === 'sending'}>
        {status === 'sending' ? 'Sending…' : submitLabel}
      </button>
      <p class="fine">
        We only use this to reply.{#if variant === 'application'}
          Max {MAX_ATTACHMENT_BYTES / 1024 / 1024} MB per file.{/if}
      </p>
    {/if}
  </form>
{/if}

<style>
  .signup {
    display: grid;
    gap: 0.75rem;
    max-width: 32rem;
    color: var(--k-fg, inherit);
    font-size: 0.875rem;
  }
  h3 {
    margin: 0;
    font-size: 1.125rem;
  }
  .blurb {
    margin: 0;
    opacity: 0.8;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  label {
    display: grid;
    gap: 0.25rem;
    flex: 1 1 12rem;
  }
  .full {
    display: grid;
    gap: 0.25rem;
  }
  span {
    font-size: 0.75rem;
    opacity: 0.8;
  }
  input,
  textarea {
    padding: 0.5rem 0.625rem;
    border: 1px solid color-mix(in srgb, currentColor 25%, transparent);
    border-radius: 0.375rem;
    background: color-mix(in srgb, currentColor 6%, transparent);
    color: inherit;
    font: inherit;
  }
  .hp {
    position: absolute;
    left: -9999px;
    width: 1px;
    height: 1px;
    overflow: hidden;
  }
  button {
    justify-self: start;
    padding: 0.5rem 0.9rem;
    border: 0;
    border-radius: 0.375rem;
    background: var(--k-accent, #f5b942);
    color: #111;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.6;
    cursor: progress;
  }
  .ok,
  .err {
    margin: 0;
  }
  .err {
    color: #f87171;
  }
  .fine {
    margin: 0;
    font-size: 0.6875rem;
    opacity: 0.6;
  }
</style>
