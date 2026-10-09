// Portable Hurl runner: avoids shell-specific syntax (`$()`, globs) so
// `mise run test:hurl` works under sh/dash/bash (CI) and pwsh (Windows dev).
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const dir = path.join('backend', 'tests', 'hurl');
const files = fs
  .readdirSync(dir)
  .filter((f) => f.endsWith('.hurl'))
  .sort()
  .map((f) => path.join(dir, f));

if (files.length === 0) {
  console.error(`No .hurl files found in ${dir}`);
  process.exit(1);
}

const runId = Date.now().toString();
const result = spawnSync(
  'hurl',
  ['--variable', 'base_url=http://localhost:8080', `--variable=run_id=${runId}`, ...files],
  { stdio: 'inherit' },
);
process.exit(result.status ?? 1);
