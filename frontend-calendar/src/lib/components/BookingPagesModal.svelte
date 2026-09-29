<script lang="ts">
  import { Copy, RefreshCw, Trash2, Plus, X, ExternalLink } from 'lucide-svelte';
  import { triggerUndoAction } from '@kestrel/shared/stores';
  import type { BookingPage } from '@kestrel/shared/api';

  let {
    open = false,
    calendars = [] as { id: string; name: string }[],
    onClose = () => {},
  } = $props<{
    open?: boolean;
    calendars?: { id: string; name: string }[];
    onClose?: () => void;
  }>();

  let pages = $state<BookingPage[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let copiedId = $state<string | null>(null);
  let showCreate = $state(false);
  let serverBase = $state('');
  let wasOpen = $state(false);

  let fCalendar = $state('');
  let fName = $state('');
  let fDuration = $state(30);
  let fBuffer = $state(0);
  let fWindow = $state(14);

  async function api() {
    return import('@kestrel/shared/api');
  }

  function setError(action: string, e: unknown) {
    error = `${action}: ${e instanceof Error ? e.message : 'failed'}`;
  }

  function onlineOrError(action: string): boolean {
    if (typeof navigator !== 'undefined' && navigator.onLine === false) {
      error = `${action}: needs a connection. Reconnect and retry.`;
      return false;
    }
    return true;
  }

  function shareLink(slug: string): string {
    return `${serverBase || 'http://localhost:8080'}/book/${slug}`;
  }

  async function reload() {
    loading = true;
    error = null;
    try {
      const { listBookingPages, getServerUrl } = await api();
      serverBase = getServerUrl();
      pages = await listBookingPages();
    } catch (e) {
      setError('Load', e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open && !wasOpen) {
      wasOpen = true;
      showCreate = false;
      reload();
    }
  });

  $effect(() => {
    if (!open && wasOpen) wasOpen = false;
  });

  $effect(() => {
    if (calendars.length > 0 && !fCalendar) fCalendar = calendars[0].id;
  });

  async function create() {
    if (!fCalendar || !fName.trim()) {
      error = 'Create: pick a calendar and enter a name.';
      return;
    }
    if (!onlineOrError('Create')) return;
    try {
      const { createBookingPage } = await api();
      const page = await createBookingPage({
        calendar_id: fCalendar,
        name: fName.trim(),
        duration_mins: fDuration,
        buffer_mins: fBuffer,
        window_days: fWindow,
      });
      pages = [...pages, page];
      fName = '';
      showCreate = false;
    } catch (e) {
      setError('Create', e);
    }
  }

  async function rotate(id: string) {
    if (!onlineOrError('Rotate')) return;
    if (!confirm('Rotate link? The current guest link dies immediately.')) return;
    try {
      const { rotateBookingSlug } = await api();
      const updated = await rotateBookingSlug(id);
      pages = pages.map((p) => (p.id === id ? updated : p));
    } catch (e) {
      setError('Rotate', e);
    }
  }

  async function toggleActive(page: BookingPage) {
    if (!onlineOrError('Update')) return;
    try {
      const { updateBookingPage } = await api();
      const updated = await updateBookingPage(page.id, {
        calendar_id: page.calendar_id,
        name: page.name,
        duration_mins: page.duration_mins,
        buffer_mins: page.buffer_mins,
        window_days: page.window_days,
        is_active: !page.is_active,
      });
      pages = pages.map((p) => (p.id === page.id ? updated : p));
    } catch (e) {
      setError('Update', e);
    }
  }

  async function remove(id: string) {
    if (!onlineOrError('Delete')) return;
    const snapshot = pages;
    const target = pages.find((p) => p.id === id);
    pages = pages.filter((p) => p.id !== id);
    triggerUndoAction({
      title: 'Booking page deleted',
      description: target ? `${target.name}: link stops working on commit.` : undefined,
      onCommit: async () => {
        const { deleteBookingPage } = await api();
        await deleteBookingPage(id);
      },
      onUndo: () => {
        pages = snapshot;
      },
      type: 'warning',
    });
  }

  async function copyLink(page: BookingPage) {
    const link = shareLink(page.slug);
    let ok = false;
    try {
      await navigator.clipboard.writeText(link);
      ok = true;
    } catch {
      ok = fallbackCopy(link);
    }
    if (ok) {
      copiedId = page.id;
      setTimeout(() => {
        if (copiedId === page.id) copiedId = null;
      }, 1500);
    } else {
      error = 'Copy: clipboard unavailable, long-press the Open link instead.';
    }
  }

  function fallbackCopy(text: string): boolean {
    try {
      const active = document.activeElement as HTMLElement | null;
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.readOnly = true;
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      const done = document.execCommand('copy');
      ta.remove();
      active?.focus?.();
      return done;
    } catch {
      return false;
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 p-4"
    role="dialog"
    aria-modal="true"
    aria-label="Booking pages"
    tabindex="-1"
    onkeydown={(e) => {
      if (e.key === 'Escape') onClose();
      e.stopPropagation();
    }}
  >
    <div class="w-full max-w-2xl max-h-[85vh] overflow-y-auto rounded-xl border border-[var(--color-border-hairline)] bg-[#131313] p-5">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-white font-semibold">Booking pages</h2>
        <button onclick={onClose} class="p-1.5 rounded hover:bg-white/10 text-neutral-400 hover:text-white" aria-label="Close">
          <X class="w-4 h-4" />
        </button>
      </div>

      {#if error}
        <p class="mb-3 text-sm text-red-400">{error}</p>
      {/if}

      {#if loading}
        <p class="text-sm text-neutral-400">Loading…</p>
      {:else}
        {#if pages.length === 0}
          <p class="text-sm text-neutral-400 mb-4">No booking pages yet. Guests book through a share link with no login.</p>
        {/if}
        <ul class="space-y-2 mb-4">
          {#each pages as page (page.id)}
            <li class="rounded-lg border border-white/10 p-3 {page.is_active ? '' : 'opacity-50'}">
              <div class="flex items-center justify-between gap-2">
                <div class="min-w-0">
                  <p class="text-sm text-white font-medium truncate">{page.name}</p>
                  <p class="text-xs text-neutral-400">{page.duration_mins} min · buffer {page.buffer_mins} min · {page.window_days}d window</p>
                </div>
                <span class="text-[10px] uppercase px-2 py-0.5 rounded-full {page.is_active ? 'bg-green-500/20 text-green-300' : 'bg-white/10 text-neutral-400'}">
                  {page.is_active ? 'live' : 'off'}
                </span>
              </div>
              <div class="mt-2 flex flex-wrap items-center gap-1.5">
                <button onclick={() => copyLink(page)} class="flex items-center gap-1 px-2 py-1 text-xs rounded bg-white/10 hover:bg-white/20 text-white" title={shareLink(page.slug)}>
                  <Copy class="w-3 h-3" /> {copiedId === page.id ? 'Copied!' : 'Copy link'}
                </button>
                <a href={shareLink(page.slug)} target="_blank" rel="noreferrer" class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-white/10 text-neutral-300" title="Open guest page">
                  <ExternalLink class="w-3 h-3" /> Open
                </a>
                <button onclick={() => toggleActive(page)} class="px-2 py-1 text-xs rounded hover:bg-white/10 text-neutral-300">
                  {page.is_active ? 'Deactivate' : 'Activate'}
                </button>
                <button onclick={() => rotate(page.id)} class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-white/10 text-neutral-300" title="New link, old link dies">
                  <RefreshCw class="w-3 h-3" /> Rotate link
                </button>
                <button onclick={() => remove(page.id)} class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-red-500/20 text-red-300" aria-label="Delete {page.name}">
                  <Trash2 class="w-3 h-3" /> Delete
                </button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}

      {#if showCreate}
        <div class="rounded-lg border border-white/10 p-3 space-y-2 mb-2">
          <label class="block text-xs text-neutral-400">Calendar
            <select bind:value={fCalendar} class="mt-1 w-full rounded bg-white/10 px-2 py-1.5 text-sm text-white">
              {#each calendars as c (c.id)}
                <option value={c.id}>{c.name}</option>
              {/each}
            </select>
          </label>
          <label class="block text-xs text-neutral-400">Name
            <input bind:value={fName} placeholder="Coffee chat" class="mt-1 w-full rounded bg-white/10 px-2 py-1.5 text-sm text-white" />
          </label>
          <div class="grid grid-cols-3 gap-2">
            <label class="block text-xs text-neutral-400">Minutes
              <input type="number" bind:value={fDuration} min="5" max="720" step="5" class="mt-1 w-full rounded bg-white/10 px-2 py-1.5 text-sm text-white" />
            </label>
            <label class="block text-xs text-neutral-400">Buffer
              <input type="number" bind:value={fBuffer} min="0" max="120" step="5" class="mt-1 w-full rounded bg-white/10 px-2 py-1.5 text-sm text-white" />
            </label>
            <label class="block text-xs text-neutral-400">Window (days)
              <input type="number" bind:value={fWindow} min="1" max="60" class="mt-1 w-full rounded bg-white/10 px-2 py-1.5 text-sm text-white" />
            </label>
          </div>
          <div class="flex gap-2">
            <button onclick={create} class="flex items-center gap-1 px-3 py-1.5 text-sm rounded bg-blue-600 hover:bg-blue-500 text-white">
              <Plus class="w-3.5 h-3.5" /> Create page
            </button>
            <button onclick={() => (showCreate = false)} class="px-3 py-1.5 text-sm rounded hover:bg-white/10 text-neutral-300">Cancel</button>
          </div>
        </div>
      {:else}
        <button onclick={() => (showCreate = true)} disabled={calendars.length === 0} class="flex items-center gap-1 px-3 py-1.5 text-sm rounded bg-white/10 hover:bg-white/20 text-white disabled:opacity-50">
          <Plus class="w-3.5 h-3.5" /> New booking page
        </button>
      {/if}
    </div>
  </div>
{/if}
