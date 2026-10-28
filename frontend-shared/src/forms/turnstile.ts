// Cloudflare Turnstile loader for the site forms. Explicit rendering keeps the
// widget scoped to one form; the token is verified server-side by the receiver.

export interface TurnstileApi {
  render(
    container: HTMLElement,
    options: { sitekey: string; callback: (token: string) => void },
  ): string;
  remove(widgetId: string): void;
}

declare global {
  interface Window {
    turnstile?: TurnstileApi;
  }
}

const SCRIPT_SRC = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';

let pending: Promise<TurnstileApi> | null = null;

export function loadTurnstile(doc: Document = document): Promise<TurnstileApi> {
  if (doc.defaultView?.turnstile) return Promise.resolve(doc.defaultView.turnstile);
  if (pending) return pending;

  pending = new Promise<TurnstileApi>((resolve, reject) => {
    const script = doc.createElement('script');
    script.src = SCRIPT_SRC;
    script.async = true;
    script.defer = true;
    script.onload = () => {
      const api = doc.defaultView?.turnstile;
      if (api) resolve(api);
      else reject(new Error('Turnstile loaded without an API'));
    };
    script.onerror = () => {
      pending = null;
      reject(new Error('Could not load Turnstile'));
    };
    doc.head.appendChild(script);
  });

  return pending;
}
