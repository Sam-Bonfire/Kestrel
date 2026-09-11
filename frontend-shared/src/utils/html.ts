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
