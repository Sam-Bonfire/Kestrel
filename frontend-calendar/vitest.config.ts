import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/lib/components/setup.ts'],
    alias: {
      '@kestrel/shared': resolve(__dirname, '../frontend-shared/src'),
      '$lib': resolve(__dirname, './src/lib')
    },
    deps: {
      inline: [/lucide-svelte/]
    }
  },
  resolve: {
    conditions: ['browser']
  }
});
