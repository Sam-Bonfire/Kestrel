import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Static single-page build, emitted to dist/ and served by the Axum backend.
export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2020',
    cssMinify: true,
    minify: 'esbuild',
    assetsInlineLimit: 8192,
  },
});
