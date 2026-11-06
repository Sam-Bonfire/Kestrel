import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

if (typeof Element !== 'undefined' && !Element.prototype.getAnimations) {
  Element.prototype.getAnimations = vi.fn(() => []);
}
