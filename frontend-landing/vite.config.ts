import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

const root = dirname(fileURLToPath(import.meta.url));

// Static single-page build, emitted to dist/ and served by the Axum backend.
// $lib points at Kestrel Mail's lib so the real ThreadList can render here.
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: {
      $lib: resolve(root, '../frontend-mail/src/lib'),
      '@kestrel/shared': resolve(root, '../frontend-shared/src'),
    },
    dedupe: ['svelte'],
  },
  optimizeDeps: {
    exclude: ['@kestrel/shared', 'frontend-mail', 'frontend-calendar'],
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2020',
    cssMinify: true,
    minify: 'esbuild',
    assetsInlineLimit: 8192,
  },
});
