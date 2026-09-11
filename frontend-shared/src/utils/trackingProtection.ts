const ALLOW_KEY = 'kestrel:privacy:remote-image-domains';
const PLACEHOLDER =
  'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';

function loadAllowed(): Set<string> {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(ALLOW_KEY);
      const parsed = raw !== null ? JSON.parse(raw) : [];
      return new Set(Array.isArray(parsed) ? parsed : []);
    }
    return new Set();
  } catch {
    return new Set();
  }
}

function saveAllowed(domains: Set<string>) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(ALLOW_KEY, JSON.stringify([...domains]));
    }
  } catch {
    // Non-fatal
  }
}

/** Lower-cased domain of an email address, '' when unparseable. */
export function senderDomain(email: string): string {
  const at = email.trim().lastIndexOf('@');
  return at >= 0 ? email.trim().slice(at + 1).toLowerCase() : '';
}

export function isDomainAllowed(domain: string): boolean {
  return loadAllowed().has(domain.toLowerCase());
}

/** Trust a sender's domain so its remote images load automatically. */
export function allowSenderDomain(email: string) {
  const domain = senderDomain(email);
  if (!domain) return;
  const allowed = loadAllowed();
  allowed.add(domain);
  saveAllowed(allowed);
}

export function blockSenderDomain(email: string) {
  const domain = senderDomain(email);
  if (!domain) return;
  const allowed = loadAllowed();
  allowed.delete(domain);
  saveAllowed(allowed);
}

function isRemoteUrl(url: string | null): boolean {
  return !!url && /^\s*(https?:)?\/\//i.test(url);
}

function hasRemoteCssUrl(css: string): boolean {
  return /url\(\s*['"]?((https?:)?\/\/)/i.test(css);
}

function stripRemoteCssUrls(css: string): string {
  return css.replace(/url\(\s*['"]?((https?:)?\/\/)[^)]+\)/gi, 'none');
}

export interface BlockedResult {
  html: string;
  blockedCount: number;
}

/**
 * Neutralize remote content (tracking pixels, remote images) in email HTML.
 * Remote <img> sources are swapped for an inline placeholder; remote
 * CSS background urls are stripped. cid: and data: images are kept.
 */
export function blockRemoteImages(html: string): BlockedResult {
  if (typeof DOMParser === 'undefined') return { html, blockedCount: 0 };
  const doc = new DOMParser().parseFromString(html, 'text/html');
  let blockedCount = 0;

  doc.querySelectorAll('img').forEach((img) => {
    if (isRemoteUrl(img.getAttribute('src'))) {
      img.setAttribute('src', PLACEHOLDER);
      img.setAttribute('data-kestrel-blocked', 'remote-image');
      if (!img.hasAttribute('alt')) img.setAttribute('alt', '[Blocked remote image]');
      blockedCount++;
    }
    const srcset = img.getAttribute('srcset') ?? '';
    if (/((https?:)?\/\/)/i.test(srcset)) img.removeAttribute('srcset');
  });

  doc.querySelectorAll<HTMLElement>('[style]').forEach((el) => {
    const style = el.getAttribute('style') ?? '';
    if (hasRemoteCssUrl(style)) {
      el.setAttribute('style', stripRemoteCssUrls(style));
      blockedCount++;
    }
  });

  // Remote urls inside <style> blocks (kept for legit email styling)
  doc.querySelectorAll('style').forEach((el) => {
    if (hasRemoteCssUrl(el.textContent ?? '')) {
      el.textContent = stripRemoteCssUrls(el.textContent ?? '');
      blockedCount++;
    }
  });

  // Legacy background attribute
  doc.querySelectorAll('[background]').forEach((el) => {
    if (isRemoteUrl(el.getAttribute('background'))) {
      el.removeAttribute('background');
      blockedCount++;
    }
  });

  return { html: doc.body.innerHTML, blockedCount };
}

/** True when the HTML references any remote image or background. */
export function hasRemoteContent(html: string): boolean {
  if (typeof DOMParser === 'undefined') return false;
  const doc = new DOMParser().parseFromString(html, 'text/html');
  for (const img of Array.from(doc.querySelectorAll('img'))) {
    if (isRemoteUrl(img.getAttribute('src'))) return true;
  }
  for (const el of Array.from(doc.querySelectorAll<HTMLElement>('[style]'))) {
    if (hasRemoteCssUrl(el.getAttribute('style') ?? '')) return true;
  }
  return false;
}
