<script lang="ts">
  import { X, Reply, Archive, Trash2, ShieldAlert, ShieldCheck } from 'lucide-svelte';

  export let thread: {
    id: string;
    sender: string;
    subject: string;
    date: string;
    htmlBody: string;
  } | null = null;

  export let onClose: () => void = () => {};

  let allowImages = false;
  let previousThreadId: string | undefined = undefined;

  $: if (thread && thread.id !== previousThreadId) {
    allowImages = false;
    previousThreadId = thread.id;
  }

  import { authState } from '@kestrel/shared';

  $: processedHtmlBody = thread ? processHtml(thread.htmlBody, allowImages) : '';

  function processHtml(html: string, allow: boolean): string {
    let processed = html;

    // Aggressively replace src attribute on img tags, matching quoted and unquoted styles
    // and protocol-relative URLs (e.g. //example.com).
    processed = processed.replace(/(<img\b[^>]*\bsrc\s*=\s*)(['"]?)((?:https?:)?\/\/[^\s>'"]+)(\2)([^>]*>)/gi, (match, beforeSrc, quote, url, quoteMatch, afterSrc) => {
      // Normalize protocol-relative URLs
      let fullUrl = url;
      if (fullUrl.startsWith('//')) {
          fullUrl = 'https:' + fullUrl;
      }
      const encodedUrl = encodeURIComponent(fullUrl);
      const proxyUrl = `/api/proxy/image?url=${encodedUrl}`;
      // Always put quotes around our injected URL to ensure valid HTML
      return `${beforeSrc}"${proxyUrl}"${afterSrc}`;
    });

    // If images are allowed, ONLY allow them from 'self' (the backend proxy), data:, blob:, and cid:.
    // This strictly prevents the browser from ever leaking the user IP directly to an external server.
    const imgSrc = allow ? "img-src 'self' data: blob: cid:;" : "img-src data: blob: cid:;";
    const csp = `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline' *; font-src * data:; ${imgSrc}">`;

    // Inject into head if it exists, otherwise prepend
    const headMatch = processed.match(/<head\b[^>]*>/i);
    if (headMatch) {
      return processed.replace(headMatch[0], `${headMatch[0]}\n${csp}`);
    }

    // Check for html tag
    const htmlMatch = processed.match(/<html\b[^>]*>/i);
    if (htmlMatch) {
      return processed.replace(htmlMatch[0], `${htmlMatch[0]}\n<head>${csp}</head>`);
    }

    return `${csp}\n${processed}`;
  }
</script>

{#if thread}
  <div class="flex-1 h-screen bg-[var(--color-canvas-card)] flex flex-col font-sans border-l border-[var(--color-border-hairline)] transition-all">
    <!-- Header Controls -->
    <div class="p-4 border-b border-[var(--color-border-hairline)] flex items-center justify-between">
      <div class="flex items-center gap-2">
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
          <Reply class="w-4 h-4" />
        </button>
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
          <Archive class="w-4 h-4" />
        </button>
        <button class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-red-400 transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>

        <div class="w-px h-4 bg-[var(--color-border-hairline)] mx-1"></div>

        <button
          onclick={() => allowImages = !allowImages}
          class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] transition-colors {allowImages ? 'text-green-400' : 'text-yellow-400'}"
          title={allowImages ? "External images allowed" : "External images blocked. Click to allow."}
        >
          {#if allowImages}
            <ShieldCheck class="w-4 h-4" />
          {:else}
            <ShieldAlert class="w-4 h-4" />
          {/if}
        </button>
      </div>

      <button onclick={onClose} class="p-1.5 rounded hover:bg-[var(--color-canvas-hover)] text-[var(--color-text-secondary)] hover:text-white transition-colors">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Subject & Meta Header -->
    <div class="p-6 border-b border-[var(--color-border-hairline)]">
      <h1 class="text-lg font-bold text-white mb-2">{thread.subject}</h1>
      <div class="flex items-center justify-between text-xs text-[var(--color-text-secondary)]">
        <span class="font-semibold text-[var(--color-text-primary)]">{thread.sender}</span>
        <span class="font-mono">{thread.date}</span>
      </div>
    </div>

    <!-- Sandboxed HTML iframe content -->
    <div class="flex-1 p-6 bg-white overflow-hidden">
      <iframe
        title="Email content"
        sandbox="allow-same-origin"
        srcdoc={processedHtmlBody}
        class="w-full h-full border-none"
      ></iframe>
    </div>
  </div>
{:else}
  <div class="flex-1 h-screen bg-[var(--color-canvas-card)] flex items-center justify-center text-xs text-[var(--color-text-secondary)] font-mono border-l border-[var(--color-border-hairline)]">
    Select a thread to view message details
  </div>
{/if}
