/**
 * Strip HTML tags for plain-text previews (grid rows, popovers).
 * Collapses whitespace; returns '' for empty input.
 */
export function plainText(html: string | null | undefined): string {
  if (!html) return '';
  const spaced = html.replace(/>\s*</g, '> <');
  if (typeof DOMParser !== 'undefined') {
    const text = new DOMParser().parseFromString(spaced, 'text/html').body.textContent ?? '';
    return text.replace(/\s+/g, ' ').trim();
  }
  return spaced.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
}

import DOMPurify from 'dompurify';

/**
 * Single sanitizer seam for all email-derived HTML rendered via {@html}.
 * Thread-history bodies, notes, and signatures cross this interface;
 * the sandboxed full-document iframe keeps its own WHOLE_DOCUMENT policy.
 */
export function sanitizeEmailBody(html: string | null | undefined): string {
  if (!html) return '';
  return DOMPurify.sanitize(html, {
    FORBID_TAGS: ['style', 'form', 'input', 'button', 'object', 'embed', 'iframe'],
    FORBID_ATTR: ['style', 'onerror', 'onload'],
  }).toString();
}

/**
 * Provider branding SVGs (backend /api/v1/providers, WASM-plugin supplied)
 * rendered via {@html} on login and settings screens. SVG profile only:
 * no event-handler attributes, no script, links neutered.
 */
export function sanitizeProviderIcon(svg: string | null | undefined): string {
  if (!svg) return '';
  return DOMPurify.sanitize(svg, {
    USE_PROFILES: { svg: true },
    FORBID_ATTR: ['onload', 'onerror', 'onclick'],
  }).toString();
}
