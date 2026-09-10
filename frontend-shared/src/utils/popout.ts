export interface PopoutDraft {
  to: string[];
  subject: string;
  body: string;
}

const DRAFT_PREFIX = 'kestrel:popout:draft:';

/** Stash a compose draft for a pop-out window; returns the handoff nonce. */
export function stagePopoutDraft(draft: PopoutDraft): string {
  const nonce = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
  // Throws on quota exhaustion so the caller falls back to in-app compose.
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(DRAFT_PREFIX + nonce, JSON.stringify(draft));
  }
  return nonce;
}

/** Take a stashed draft exactly once; null when missing or consumed. */
export function takePopoutDraft(nonce: string): PopoutDraft | null {
  try {
    if (typeof localStorage === 'undefined') return null;
    const raw = localStorage.getItem(DRAFT_PREFIX + nonce);
    localStorage.removeItem(DRAFT_PREFIX + nonce);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') return null;
    return {
      to: Array.isArray(parsed.to) ? parsed.to : [],
      subject: typeof parsed.subject === 'string' ? parsed.subject : '',
      body: typeof parsed.body === 'string' ? parsed.body : '',
    };
  } catch {
    return null;
  }
}

/** True when this window was opened as a compose pop-out. */
export function popoutDraftNonce(href?: string): string | null {
  try {
    const url = new URL(href ?? (typeof window !== 'undefined' ? window.location.href : ''));
    if (url.searchParams.get('popout') !== 'compose') return null;
    return url.searchParams.get('draft');
  } catch {
    return null;
  }
}

export function buildPopoutUrl(nonce: string): string {
  return `?popout=compose&draft=${encodeURIComponent(nonce)}`;
}

/**
 * Open a compose pop-out: a Tauri window when available,
 * a browser tab otherwise.
 */
export async function openComposePopout(draft: PopoutDraft): Promise<void> {
  const nonce = stagePopoutDraft(draft);
  const url = buildPopoutUrl(nonce);
  if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    new WebviewWindow(`compose-${nonce}`, { url, title: 'Compose', width: 720, height: 640 });
    return;
  }
  // Throws (popup blocker) so the caller falls back to in-app compose.
  const popup = window.open(url, '_blank', 'width=720,height=640');
  if (!popup) throw new Error('Pop-up blocked by the browser');
}
