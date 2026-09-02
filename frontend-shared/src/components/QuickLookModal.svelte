<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import {
    X,
    ZoomIn,
    ZoomOut,
    RotateCcw,
    ChevronLeft,
    ChevronRight,
    Download,
    File,
    FileImage,
    FileText,
    FileArchive,
    FileCode,
    FileSpreadsheet,
    Paperclip
  } from 'lucide-svelte';
  import type { PDFDocumentProxy, RenderTask } from 'pdfjs-dist';

  export interface QuickLookAttachment {
    filename: string;
    size: number;
    contentType?: string;
    data?: Uint8Array | ArrayBuffer | string | null;
  }

  let {
    isOpen = false,
    attachment = null as QuickLookAttachment | null,
    onClose = () => {},
    onDownload = () => {}
  } = $props<{
    isOpen?: boolean;
    attachment?: QuickLookAttachment | null;
    onClose?: () => void;
    onDownload?: (att: QuickLookAttachment) => void;
  }>();

  // Active element tracking for focus restoration
  let previousActiveElement: HTMLElement | null = null;
  let modalContainer: HTMLDivElement | null = $state(null);

  // Object URL state & management
  let currentObjectUrl = $state<string | null>(null);

  function cleanupObjectUrl() {
    if (currentObjectUrl) {
      URL.revokeObjectURL(currentObjectUrl);
      currentObjectUrl = null;
    }
  }

  onDestroy(() => {
    cleanupObjectUrl();
    destroyPdf();
  });

  // Derived attachment details
  const extension = $derived(
    attachment?.filename ? attachment.filename.split('.').pop()?.toLowerCase() || '' : ''
  );

  const isImage = $derived.by(() => {
    if (!attachment) return false;
    const ct = attachment.contentType?.toLowerCase() || '';
    if (ct.startsWith('image/')) return true;
    const imgExts = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'bmp'];
    return imgExts.includes(extension);
  });

  const isPdf = $derived.by(() => {
    if (!attachment) return false;
    const ct = attachment.contentType?.toLowerCase() || '';
    if (ct === 'application/pdf') return true;
    return extension === 'pdf';
  });

  // Image Preview State
  let imageZoom = $state(1);

  function handleZoomIn() {
    imageZoom = Math.min(3, +(imageZoom + 0.25).toFixed(2));
  }

  function handleZoomOut() {
    imageZoom = Math.max(0.25, +(imageZoom - 0.25).toFixed(2));
  }

  function handleResetZoom() {
    imageZoom = 1;
  }

  function handleImageDoubleClick() {
    if (imageZoom === 1) {
      imageZoom = 1.75;
    } else {
      imageZoom = 1;
    }
  }

  // PDF Preview State using pdfjs-dist
  let pdfCanvas = $state<HTMLCanvasElement | null>(null);
  let pdfDoc: PDFDocumentProxy | null = null;
  let pdfCurrentPage = $state(1);
  let pdfJumpInput = $state('1');
  let pdfTotalPages = $state(0);
  let pdfScale = $state(1);
  let pdfLoading = $state(false);
  let pdfError = $state<string | null>(null);
  let currentRenderTask: RenderTask | null = null;

  function destroyPdf() {
    if (currentRenderTask) {
      try {
        currentRenderTask.cancel();
      } catch {}
      currentRenderTask = null;
    }
    if (pdfDoc) {
      try {
        pdfDoc.destroy();
      } catch {}
      pdfDoc = null;
    }
    pdfCurrentPage = 1;
    pdfTotalPages = 0;
    pdfScale = 1;
    pdfLoading = false;
    pdfError = null;
  }

  async function loadPdfDocument(data: Uint8Array | ArrayBuffer) {
    pdfLoading = true;
    pdfError = null;
    try {
      const pdfjs = await import('pdfjs-dist');
      if (!pdfjs.GlobalWorkerOptions.workerSrc) {
        // Fallback worker URL for pdfjs-dist
        pdfjs.GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjs.version}/pdf.worker.min.mjs`;
      }
      const loadingTask = pdfjs.getDocument({ data });
      pdfDoc = await loadingTask.promise;
      pdfTotalPages = pdfDoc.numPages;
      pdfCurrentPage = 1;
      pdfJumpInput = '1';
      pdfLoading = false;
    } catch (err: unknown) {
      console.error('Failed to load PDF preview:', err);
      const errMsg = err instanceof Error ? err.message : 'Failed to load PDF preview';
      pdfError = errMsg;
      pdfLoading = false;
    }
  }

  async function renderPdfPage(pageNumber: number) {
    if (!pdfDoc || !pdfCanvas) return;
    try {
      if (currentRenderTask) {
        try {
          currentRenderTask.cancel();
        } catch {}
        currentRenderTask = null;
      }
      const page = await pdfDoc.getPage(pageNumber);
      const dpr = window.devicePixelRatio || 1;
      const viewport = page.getViewport({ scale: pdfScale * dpr });

      pdfCanvas.height = viewport.height;
      pdfCanvas.width = viewport.width;
      pdfCanvas.style.height = `${viewport.height / dpr}px`;
      pdfCanvas.style.width = `${viewport.width / dpr}px`;

      const ctx = pdfCanvas.getContext('2d');
      if (!ctx) return;

      const renderContext = {
        canvasContext: ctx,
        viewport
      };

      currentRenderTask = page.render(renderContext);
      await currentRenderTask.promise;
      currentRenderTask = null;
    } catch (err: unknown) {
      if (err instanceof Error && err.name !== 'RenderingCancelledException') {
        console.error('Error rendering PDF page:', err);
      }
    }
  }

  function handlePageJumpSubmit(e: SubmitEvent) {
    e.preventDefault();
    const pageNo = parseInt(pdfJumpInput, 10);
    if (!isNaN(pageNo) && pageNo >= 1 && pageNo <= pdfTotalPages) {
      pdfCurrentPage = pageNo;
    } else {
      pdfJumpInput = String(pdfCurrentPage);
    }
  }

  // Handle data updates & object URL generation
  $effect(() => {
    const active = isOpen;
    const att = attachment;

    untrack(() => {
      if (active && att) {
        // Save focused element for restoration on close
        if (document.activeElement instanceof HTMLElement && !previousActiveElement) {
          previousActiveElement = document.activeElement;
        }

        cleanupObjectUrl();
        destroyPdf();
        imageZoom = 1;

        if (att.data) {
          let blob: Blob;
          let arrayBuf: ArrayBuffer | null = null;
          let generatedUrl: string | null = null;

          if (att.data instanceof Uint8Array) {
            arrayBuf = att.data.buffer.slice(
              att.data.byteOffset,
              att.data.byteOffset + att.data.byteLength
            ) as ArrayBuffer;
            blob = new Blob([att.data], { type: att.contentType || 'application/octet-stream' });
          } else if (att.data instanceof ArrayBuffer) {
            arrayBuf = att.data;
            blob = new Blob([att.data], { type: att.contentType || 'application/octet-stream' });
          } else if (typeof att.data === 'string') {
            if (att.data.startsWith('data:') || att.data.startsWith('blob:') || att.data.startsWith('http')) {
              generatedUrl = att.data;
              blob = new Blob();
            } else {
              const encoder = new TextEncoder();
              const u8 = encoder.encode(att.data);
              arrayBuf = u8.buffer as ArrayBuffer;
              blob = new Blob([u8], { type: att.contentType || 'text/plain' });
            }
          } else {
            blob = new Blob();
          }

          if (generatedUrl) {
            currentObjectUrl = generatedUrl;
          } else if (blob.size > 0) {
            currentObjectUrl = URL.createObjectURL(blob);
          }

          if (isPdf && arrayBuf) {
            loadPdfDocument(arrayBuf);
          }
        }

        setTimeout(() => modalContainer?.focus(), 50);
      } else {
        cleanupObjectUrl();
        destroyPdf();
        if (previousActiveElement) {
          previousActiveElement.focus();
          previousActiveElement = null;
        }
      }
    });
  });

  // Re-render PDF on page or scale change
  $effect(() => {
    if (pdfDoc && pdfCanvas && pdfCurrentPage && pdfScale) {
      pdfJumpInput = String(pdfCurrentPage);
      renderPdfPage(pdfCurrentPage);
    }
  });

  function handleCloseModal() {
    cleanupObjectUrl();
    destroyPdf();
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.stopPropagation();
      handleCloseModal();
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes <= 0 || isNaN(bytes)) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  }

  function getFallbackIcon() {
    if (['zip', 'tar', 'gz', 'rar', '7z'].includes(extension)) return FileArchive;
    if (['csv', 'xlsx', 'xls'].includes(extension)) return FileSpreadsheet;
    if (['js', 'ts', 'json', 'html', 'css', 'py', 'rs'].includes(extension)) return FileCode;
    return File;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen && attachment}
  <div
    transition:fade={{ duration: 150 }}
    class="fixed inset-0 z-50 flex flex-col bg-black/80 backdrop-blur-md select-none outline-none"
    role="dialog"
    aria-modal="true"
    aria-label="Attachment Preview"
    tabindex="-1"
    bind:this={modalContainer}
    onclick={handleCloseModal}
  >
    <!-- Header Bar -->
    <div
      class="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-neutral-900/80 text-white shrink-0"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center gap-3 min-w-0">
        <div class="w-8 h-8 rounded-lg bg-purple-500/20 text-purple-400 flex items-center justify-center shrink-0">
          <Paperclip class="w-4 h-4" />
        </div>
        <div class="flex flex-col min-w-0">
          <span class="text-sm font-semibold truncate text-white">{attachment.filename}</span>
          <span class="text-xs text-neutral-400">{formatBytes(attachment.size)}</span>
        </div>
      </div>

      <!-- Controls Toolbar -->
      <div class="flex items-center gap-2">
        {#if isImage}
          <div class="flex items-center gap-1 bg-white/5 border border-white/10 rounded-lg p-1 text-xs">
            <button
              type="button"
              onclick={handleZoomOut}
              class="p-1.5 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer"
              title="Zoom out"
              aria-label="Zoom out image"
            >
              <ZoomOut class="w-4 h-4" />
            </button>
            <span class="px-2 font-mono text-neutral-300">{Math.round(imageZoom * 100)}%</span>
            <button
              type="button"
              onclick={handleZoomIn}
              class="p-1.5 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer"
              title="Zoom in"
              aria-label="Zoom in image"
            >
              <ZoomIn class="w-4 h-4" />
            </button>
            <button
              type="button"
              onclick={handleResetZoom}
              class="p-1.5 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer"
              title="Reset Zoom"
              aria-label="Reset image zoom"
            >
              <RotateCcw class="w-3.5 h-3.5" />
            </button>
          </div>
        {:else if isPdf && pdfTotalPages > 0}
          <div class="flex items-center gap-2 bg-white/5 border border-white/10 rounded-lg p-1 text-xs">
            <button
              type="button"
              disabled={pdfCurrentPage <= 1}
              onclick={() => pdfCurrentPage = Math.max(1, pdfCurrentPage - 1)}
              class="p-1.5 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer disabled:opacity-30"
              title="Previous Page"
              aria-label="Previous page"
            >
              <ChevronLeft class="w-4 h-4" />
            </button>
            <form onsubmit={handlePageJumpSubmit} class="flex items-center gap-1">
              <span class="text-neutral-400">Page</span>
              <input
                type="text"
                bind:value={pdfJumpInput}
                class="w-10 bg-black/40 border border-white/20 rounded px-1 py-0.5 text-center font-mono text-white text-xs outline-none focus:border-blue-500"
                aria-label="Jump to page"
              />
              <span class="text-neutral-400">/ {pdfTotalPages}</span>
            </form>
            <button
              type="button"
              disabled={pdfCurrentPage >= pdfTotalPages}
              onclick={() => pdfCurrentPage = Math.min(pdfTotalPages, pdfCurrentPage + 1)}
              class="p-1.5 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer disabled:opacity-30"
              title="Next Page"
              aria-label="Next page"
            >
              <ChevronRight class="w-4 h-4" />
            </button>
            <div class="w-px h-4 bg-white/10 mx-1"></div>
            <button
              type="button"
              onclick={() => pdfScale = Math.max(0.5, +(pdfScale - 0.25).toFixed(2))}
              class="p-1 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer"
              title="PDF Zoom Out"
              aria-label="PDF Zoom Out"
            >
              <ZoomOut class="w-3.5 h-3.5" />
            </button>
            <span class="font-mono text-neutral-300 text-[11px]">{Math.round(pdfScale * 100)}%</span>
            <button
              type="button"
              onclick={() => pdfScale = Math.min(2.5, +(pdfScale + 0.25).toFixed(2))}
              class="p-1 rounded hover:bg-white/10 text-neutral-300 hover:text-white transition-colors cursor-pointer"
              title="PDF Zoom In"
              aria-label="PDF Zoom In"
            >
              <ZoomIn class="w-3.5 h-3.5" />
            </button>
          </div>
        {/if}

        <!-- Download Action -->
        <button
          type="button"
          onclick={() => attachment && onDownload(attachment)}
          class="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors cursor-pointer"
          title="Download File"
        >
          <Download class="w-3.5 h-3.5" />
          <span>Download</span>
        </button>

        <!-- Close Button -->
        <button
          type="button"
          onclick={handleCloseModal}
          class="p-2 rounded-lg bg-white/10 hover:bg-white/20 text-white transition-colors cursor-pointer"
          title="Close overlay (Esc)"
          aria-label="Close modal"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Main Content Display Area -->
    <div
      class="flex-1 overflow-auto flex items-center justify-center p-4 md:p-8"
      onclick={(e) => e.stopPropagation()}
    >
      {#if isImage}
        {#if currentObjectUrl}
          <div class="max-w-full max-h-full flex items-center justify-center overflow-auto">
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <img
              src={currentObjectUrl}
              alt={attachment.filename}
              ondblclick={handleImageDoubleClick}
              style="transform: scale({imageZoom}); transform-origin: center center; transition: transform 0.15s ease-out;"
              class="max-w-full max-h-[80vh] object-contain rounded shadow-2xl cursor-zoom-in"
            />
          </div>
        {:else}
          <div class="text-neutral-400 text-sm">Loading image preview...</div>
        {/if}

      {:else if isPdf}
        <div class="flex flex-col items-center justify-center w-full h-full overflow-auto">
          {#if pdfLoading}
            <div class="text-neutral-400 text-sm animate-pulse">Loading PDF document...</div>
          {:else if pdfError}
            <div class="p-6 bg-red-500/10 border border-red-500/20 rounded-xl text-center max-w-md">
              <span class="text-red-400 text-sm font-medium">{pdfError}</span>
            </div>
          {:else}
            <div class="overflow-auto max-w-full max-h-[82vh] bg-neutral-900 border border-white/10 rounded-lg shadow-2xl p-2">
              <canvas bind:this={pdfCanvas} class="block mx-auto rounded"></canvas>
            </div>
          {/if}
        </div>

      {:else}
        <!-- Fallback Card for Non-previewable Files -->
        {#key attachment.filename}
          {@const FallbackIcon = getFallbackIcon()}
          <div
            transition:scale={{ duration: 150, start: 0.95 }}
            class="w-full max-w-md bg-neutral-900 border border-white/10 rounded-2xl p-8 flex flex-col items-center text-center shadow-2xl space-y-4"
          >
            <div class="w-16 h-16 rounded-2xl bg-purple-500/10 text-purple-400 flex items-center justify-center">
              <FallbackIcon class="w-8 h-8" />
            </div>

            <div class="space-y-1 w-full">
              <h3 class="text-base font-semibold text-white truncate px-2" title={attachment.filename}>
                {attachment.filename}
              </h3>
              <p class="text-xs text-neutral-400">
                {formatBytes(attachment.size)} • {extension.toUpperCase() || 'Binary'} File
              </p>
            </div>

            <p class="text-xs text-neutral-500 max-w-xs">
              No live preview available for this file type. Download to view using an external application.
            </p>

            <button
              type="button"
              onclick={() => attachment && onDownload(attachment)}
              class="w-full flex items-center justify-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl font-medium text-xs transition-colors cursor-pointer shadow-lg active:scale-95"
            >
              <Download class="w-4 h-4" />
              <span>Download File</span>
            </button>
          </div>
        {/key}
      {/if}
    </div>
  </div>
{/if}
