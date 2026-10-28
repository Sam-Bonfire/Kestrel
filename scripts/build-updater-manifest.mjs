#!/usr/bin/env node
// Builds the Tauri updater manifests (`kestrel-mail-latest.json`,
// `kestrel-calendar-latest.json`) from the desktop artifacts of a release run.
// The updater endpoints in each tauri.conf.json point at these files.
//
// Usage:
//   node scripts/build-updater-manifest.mjs <artifactsDir> <tag> <owner/repo> <outDir>
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const APPS = {
  mail: { product: 'Kestrel.Mail', manifest: 'kestrel-mail-latest.json' },
  calendar: { product: 'Kestrel.Calendar', manifest: 'kestrel-calendar-latest.json' },
};

// Only formats the updater can install in place. The .msi is deliberately
// absent: NSIS is the updater-integrated Windows installer.
const TARGETS = [
  { suffix: '_x64-setup.exe', platform: 'windows-x86_64' },
  { suffix: '_amd64.AppImage', platform: 'linux-x86_64' },
];

/**
 * @param {{name: string, signature: string}[]} entries
 * @param {{version: string, tag: string, repo: string, pubDate: string}} opts
 * @returns {Record<string, object>} manifest file name -> manifest contents
 */
export function buildUpdaterManifests(entries, opts) {
  const manifests = {};
  for (const [app, cfg] of Object.entries(APPS)) {
    /** @type {Record<string, {signature: string, url: string}>} */
    const platforms = {};
    for (const target of TARGETS) {
      const asset = entries.find(
        (e) => e.name.startsWith(`${cfg.product}_`) && e.name.endsWith(target.suffix),
      );
      if (!asset) continue;
      platforms[target.platform] = {
        signature: asset.signature,
        url: `https://github.com/${opts.repo}/releases/download/${opts.tag}/${encodeURIComponent(asset.name)}`,
      };
    }
    if (Object.keys(platforms).length === 0) {
      throw new Error(`no updater artifacts (exe/AppImage + .sig) found for ${cfg.product}`);
    }
    manifests[cfg.manifest] = {
      version: opts.version,
      notes: `Kestrel ${opts.tag}`,
      pub_date: opts.pubDate,
      platforms,
    };
  }
  return manifests;
}

/**
 * Pairs each published installer with its signature file. A missing signature
 * fails the release: an unsigned entry makes every client reject the update.
 * @param {string[]} fileNames
 * @param {(name: string) => string} readSignature
 * @returns {{name: string, signature: string}[]}
 */
export function collectSignedAssets(fileNames, readSignature) {
  const installers = fileNames.filter((name) => TARGETS.some((t) => name.endsWith(t.suffix)));
  return installers.map((name) => {
    if (!fileNames.includes(`${name}.sig`)) {
      throw new Error(`missing signature for ${name}`);
    }
    const signature = readSignature(`${name}.sig`).trim();
    if (!signature) throw new Error(`empty signature for ${name}`);
    return { name, signature };
  });
}

function main(argv) {
  const [artifactsDir, tag, repo, outDir] = argv;
  if (!artifactsDir || !tag || !repo || !outDir) {
    console.error('usage: build-updater-manifest.mjs <artifactsDir> <tag> <owner/repo> <outDir>');
    process.exit(1);
  }
  const files = readdirSync(artifactsDir);
  const entries = collectSignedAssets(files, (name) =>
    readFileSync(join(artifactsDir, name), 'utf8'),
  );
  // The manifest version must match the version the installed binary reports,
  // which is what tauri.conf.json carries into the bundle.
  const version = JSON.parse(
    readFileSync(join('frontend-mail', 'src-tauri', 'tauri.conf.json'), 'utf8'),
  ).version;
  if (`v${version}` !== tag) {
    console.warn(`warning: bundle version ${version} does not match release tag ${tag}`);
  }
  const manifests = buildUpdaterManifests(entries, {
    version,
    tag,
    repo,
    pubDate: new Date().toISOString(),
  });
  for (const [name, manifest] of Object.entries(manifests)) {
    writeFileSync(join(outDir, name), `${JSON.stringify(manifest, null, 2)}\n`);
    console.log(`wrote ${name}`);
  }
}

if (process.argv[1] && process.argv[1].endsWith('build-updater-manifest.mjs')) {
  main(process.argv.slice(2));
}
