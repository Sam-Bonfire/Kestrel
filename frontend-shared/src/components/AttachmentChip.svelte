<script lang="ts">
  import {
    Paperclip,
    Eye,
    Download,
    FileImage,
    FileText,
    FileCode,
    FileArchive,
    FileSpreadsheet,
    File
  } from 'lucide-svelte';

  export interface AttachmentInfo {
    filename: string;
    size: number;
    contentType?: string;
  }

  let {
    attachment,
    onPreview,
    onDownload,
    downloading = false
  } = $props<{
    attachment: AttachmentInfo;
    onPreview?: (attachment: AttachmentInfo) => void;
    onDownload?: (attachment: AttachmentInfo) => void;
    downloading?: boolean;
  }>();

  function formatBytes(bytes: number): string {
    if (bytes <= 0 || isNaN(bytes)) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  }

  const extension = $derived(
    attachment?.filename ? attachment.filename.split('.').pop()?.toLowerCase() || '' : ''
  );

  const isPreviewable = $derived.by(() => {
    const ct = attachment?.contentType?.toLowerCase() || '';
    if (ct.startsWith('image/') || ct === 'application/pdf') return true;
    const imgExts = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'bmp'];
    return imgExts.includes(extension) || extension === 'pdf';
  });

  function getFileIcon() {
    const ct = attachment?.contentType?.toLowerCase() || '';
    if (ct.startsWith('image/') || ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg'].includes(extension)) {
      return FileImage;
    }
    if (ct === 'application/pdf' || extension === 'pdf' || extension === 'txt' || extension === 'md') {
      return FileText;
    }
    if (['zip', 'tar', 'gz', 'rar', '7z'].includes(extension)) {
      return FileArchive;
    }
    if (['csv', 'xlsx', 'xls'].includes(extension)) {
      return FileSpreadsheet;
    }
    if (['js', 'ts', 'json', 'html', 'css', 'py', 'rs'].includes(extension)) {
      return FileCode;
    }
    return Paperclip;
  }

  const IconComponent = $derived(getFileIcon());
</script>

<div
  class="inline-flex items-center gap-2 px-3 py-2 bg-[var(--color-canvas-card)] border border-[var(--color-border-hairline)] rounded-lg text-left transition-colors hover:bg-[var(--color-canvas-hover)] max-w-xs group"
>
  <div class="w-8 h-8 rounded bg-purple-500/10 flex items-center justify-center text-purple-400 shrink-0">
    <IconComponent class="w-4 h-4" />
  </div>

  <div class="flex flex-col min-w-0 flex-1">
    <span class="text-xs font-semibold text-white truncate" title={attachment.filename}>
      {attachment.filename}
    </span>
    <span class="text-[10px] text-[var(--color-text-secondary)]">
      {formatBytes(attachment.size)}
    </span>
  </div>

  <div class="flex items-center gap-1 shrink-0 ml-1">
    {#if onPreview && isPreviewable}
      <button
        type="button"
        onclick={(e) => {
          e.stopPropagation();
          onPreview?.(attachment);
        }}
        class="p-1.5 rounded-md hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer"
        title="Preview"
        aria-label="Preview attachment"
      >
        <Eye class="w-3.5 h-3.5" />
      </button>
    {/if}

    {#if onDownload}
      <button
        type="button"
        onclick={(e) => {
          e.stopPropagation();
          onDownload?.(attachment);
        }}
        disabled={downloading}
        class="p-1.5 rounded-md hover:bg-white/10 text-[var(--color-text-secondary)] hover:text-white transition-colors cursor-pointer disabled:opacity-50"
        title="Download"
        aria-label="Download attachment"
      >
        <Download class="w-3.5 h-3.5" />
      </button>
    {/if}
  </div>
</div>
