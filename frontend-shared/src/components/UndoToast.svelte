<script lang="ts">
  import { activeToasts, executeUndo, dismissToast } from '../stores/undoToast.js';
  import { RotateCcw, X, CheckCircle2, AlertTriangle, Info, AlertCircle } from 'lucide-svelte';
  import { fly } from 'svelte/transition';

  // Global 'Z' / 'Ctrl+Z' / 'Cmd+Z' shortcut listener for Undo
  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (
      target &&
      (target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable)
    ) {
      return; // Respect input guard
    }

    if (e.key === 'z' || e.key === 'Z') {
      if ($activeToasts.length > 0) {
        e.preventDefault();
        executeUndo();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $activeToasts.length > 0}
  <div class="fixed bottom-5 right-5 z-50 flex flex-col-reverse gap-2.5 max-w-sm w-full pointer-events-none select-none">
    {#each $activeToasts as toast (toast.id)}
      {@const progressPercent = Math.max(0, Math.min(100, (toast.remainingMs / toast.timeoutMs) * 100))}
      <div
        transition:fly={{ y: 20, duration: 200 }}
        class="pointer-events-auto relative overflow-hidden flex flex-col bg-[#18181b] border border-neutral-800 text-neutral-200 rounded-xl shadow-2xl backdrop-blur-md px-3.5 py-3"
        role="status"
        aria-live="polite"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2.5 min-w-0 flex-1">
            {#if toast.type === 'success'}
              <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0" />
            {:else if toast.type === 'warning'}
              <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0" />
            {:else if toast.type === 'error'}
              <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
            {:else}
              <Info class="w-4 h-4 text-blue-400 shrink-0" />
            {/if}

            <div class="flex flex-col min-w-0">
              <span class="text-xs font-semibold text-white truncate">{toast.title}</span>
              {#if toast.description}
                <span class="text-[11px] text-neutral-400 truncate">{toast.description}</span>
              {/if}
            </div>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <button
              type="button"
              onclick={() => executeUndo(toast.id)}
              class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-white/10 hover:bg-white/20 active:scale-95 text-xs font-semibold text-white transition-all cursor-pointer"
              title="Undo action (Z)"
            >
              <RotateCcw class="w-3 h-3 text-neutral-300" />
              <span>Undo</span>
              <kbd class="text-[9px] bg-black/40 px-1 py-0.5 rounded text-neutral-400 font-mono">Z</kbd>
            </button>

            <button
              type="button"
              onclick={() => dismissToast(toast.id)}
              class="p-1 rounded-lg hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors cursor-pointer"
              title="Dismiss"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <!-- Remaining time progress bar -->
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-neutral-800">
          <div
            class="h-full bg-blue-500 transition-all duration-75 ease-linear"
            style="width: {progressPercent}%;"
          ></div>
        </div>
      </div>
    {/each}
  </div>
{/if}
