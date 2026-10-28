import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'KestrelShared',
      fileName: (format) => `shared.${format === 'es' ? 'js' : 'cjs'}`,
      formats: ['es', 'cjs'],
    },
    rollupOptions: {
      external: [],
    },
  },
  plugins: [
    svelte(),
    dts({
      insertTypesEntry: true,
      include: ['src/**/*'],
    }),
  ],
  // Tests mount components, so resolve Svelte's browser build instead of the
  // server entry (`mount` is unavailable there). Builds use tsc, not vite.
  resolve: {
    conditions: ['browser'],
  },
  test: {
    environment: 'jsdom',
  },
});
