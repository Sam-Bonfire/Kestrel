// E2E runner: real backend behind Playwright.
// Usage: (build bins + frontends first) -> node scripts/run-e2e-backend.mjs
// Mirrors the CI `e2e-backend` job; used by `mise run test:e2e:backend`.
// Only the sqlite fixtures are supported: backend/src/bin/seed.rs speaks the
// sqlite '?' placeholder dialect, so postgres E2E waits on dialect-aware seeding.
//
// sqlx-cli is deliberately NOT used: its URL parsing disagrees with the
// workspace binaries on Windows (same `sqlite:` string resolves to a
// different file). Instead the server creates the schema itself at boot
// (pool.rs: create_if_missing + migrate!) and seed.exe then fills it — the
// single-colon absolute path below is the only form both agree on.
import { spawn, spawnSync } from 'node:child_process';
import { rmSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import process from 'node:process';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BACKEND_DIR = path.join(ROOT, 'backend');
const EXE = process.platform === 'win32' ? '.exe' : '';
const DEBUG_DIR = path.join(ROOT, 'target', 'debug');

const PORT = String(process.env.E2E_BACKEND_PORT ?? 8080);
const DB_PATH = path.join(BACKEND_DIR, 'data', 'e2e.db');
// Single `sqlite:` colon + forward slashes: pool.rs strips "sqlite:" and uses
// the remainder as a raw filename; seed.exe's URL parser resolves this form
// to the same file (backslashes and query params both break seed's parser).
const DATABASE_URL = process.env.E2E_DATABASE_URL ?? `sqlite:${DB_PATH.replaceAll('\\', '/')}`;
const PASSWORD = process.env.KESTREL_E2E_PASSWORD ?? process.env.KESTREL_SEED_PASSWORD ?? 'e2e-seed-password';

function run(cmd, args, env = {}, cwd = ROOT) {
  const result = spawnSync(cmd, args, {
    cwd,
    stdio: 'inherit',
    env: { ...process.env, ...env },
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} exited with ${result.status}`);
  }
}

async function waitForHealth(url, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // server not up yet
    }
    if (Date.now() > deadline) throw new Error(`backend never became healthy at ${url}`);
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

if (!process.env.SKIP_BUILD) {
  run('cargo', ['build', '--manifest-path', path.join('backend', 'Cargo.toml'), '--bins']);
}
mkdirSync(path.join(BACKEND_DIR, 'data'), { recursive: true });
for (const suffix of ['', '-wal', '-shm']) {
  try {
    rmSync(DB_PATH + suffix, { force: true });
  } catch (err) {
    throw new Error(`could not remove stale e2e database ${DB_PATH + suffix}: ${err.message}`);
  }
}

const seedEnv = {
  DATABASE_URL,
  KESTREL_SEED_PASSWORD: PASSWORD,
  KESTREL_SEED_E2E: '1',
};

const serverBin = path.join(DEBUG_DIR, `kestrel-server${EXE}`);
const server = spawn(serverBin, [], {
  cwd: BACKEND_DIR,
  stdio: 'inherit',
  env: { ...process.env, ...seedEnv, PORT, EXTRA_ALLOWED_ORIGINS: 'http://localhost:1423,http://localhost:1424' },
});
server.on('error', (err) => {
  console.error(`failed to start backend: ${err}`);
  process.exit(1);
});

try {
  // Health implies migrations ran: init_pool + migrate! finish before bind.
  await waitForHealth(`http://127.0.0.1:${PORT}/api/health`, 90_000);

  const seedBin = path.join(DEBUG_DIR, `seed${EXE}`);
  run(seedBin, [], seedEnv);

  const tests = spawn('pnpm', ['--filter', '@kestrel/e2e', 'exec', 'playwright', 'test', '--project=mail', '--grep', '@backend'], {
    cwd: ROOT,
    stdio: 'inherit',
    env: {
      ...process.env,
      E2E_BACKEND: '1',
      KESTREL_E2E_PASSWORD: PASSWORD,
    },
  });
  await new Promise((resolve, reject) => {
    tests.on('exit', (code) => (code === 0 ? resolve() : reject(new Error(`playwright exited with ${code}`))));
    tests.on('error', reject);
  });
} finally {
  if (!server.killed) {
    server.kill();
  }
}
