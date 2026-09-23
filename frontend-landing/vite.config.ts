import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { execSync } from 'node:child_process';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

const root = dirname(fileURLToPath(import.meta.url));

// Build-time env with fallback (empty string = feature off / derive default).
function envOr(name: string, fallback: string): string {
  const v = process.env[name]?.trim();
  return v ? v : fallback;
}

// Release tag baked in at build time (KESTREL_VERSION wins, e.g. CI build args).
// Empty when unknown (e.g. Docker build without git metadata) — the page omits
// the version instead of printing a stale one.
function kestrelVersion(): string {
  const fromEnv = process.env.KESTREL_VERSION?.trim();
  if (fromEnv) return fromEnv;
  try {
    return execSync('git describe --tags --abbrev=0', {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return '';
  }
}

// Static single-page build, emitted to dist/ and served by the Axum backend.
// $lib points at Kestrel Mail's lib so the real ThreadList can render here.
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  define: {
    __KESTREL_VERSION__: JSON.stringify(kestrelVersion()),
    // Site identity for other self-hosters (defaults live in src/site.ts).
    __KESTREL_REPO__: JSON.stringify(envOr('KESTREL_REPO', 'Sam-Bonfire/Kestrel')),
    __KESTREL_DOCKER_IMAGE__: JSON.stringify(envOr('KESTREL_DOCKER_IMAGE', '')),
    __KESTREL_CONTACT_EMAIL__: JSON.stringify(envOr('KESTREL_CONTACT_EMAIL', '')),
    __KESTREL_IOS_TESTFLIGHT__: JSON.stringify(envOr('KESTREL_IOS_TESTFLIGHT', '')),
  },
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
