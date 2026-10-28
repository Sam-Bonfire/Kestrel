import assert from 'node:assert/strict';
import { test } from 'node:test';
import { buildUpdaterManifests, collectSignedAssets } from './build-updater-manifest.mjs';

const opts = { version: '0.1.0', tag: 'v0.3.4', repo: 'Sam-Bonfire/Kestrel', pubDate: '2026-10-28T00:00:00.000Z' };

const entries = [
  { name: 'Kestrel.Mail_0.1.0_x64-setup.exe', signature: 'mail-win' },
  { name: 'Kestrel.Mail_0.1.0_amd64.AppImage', signature: 'mail-linux' },
  { name: 'Kestrel.Calendar_0.1.0_x64-setup.exe', signature: 'cal-win' },
];

test('maps installers to updater platforms and release URLs', () => {
  const manifests = buildUpdaterManifests(entries, opts);
  assert.deepEqual(manifests['kestrel-mail-latest.json'].platforms, {
    'windows-x86_64': {
      signature: 'mail-win',
      url: 'https://github.com/Sam-Bonfire/Kestrel/releases/download/v0.3.4/Kestrel.Mail_0.1.0_x64-setup.exe',
    },
    'linux-x86_64': {
      signature: 'mail-linux',
      url: 'https://github.com/Sam-Bonfire/Kestrel/releases/download/v0.3.4/Kestrel.Mail_0.1.0_amd64.AppImage',
    },
  });
  assert.equal(manifests['kestrel-calendar-latest.json'].version, '0.1.0');
  assert.deepEqual(Object.keys(manifests['kestrel-calendar-latest.json'].platforms), ['windows-x86_64']);
});

test('refuses an app with no updater artifacts', () => {
  assert.throws(
    () => buildUpdaterManifests(entries.slice(0, 2), opts),
    /no updater artifacts .*Kestrel\.Calendar/,
  );
});

test('pairs installers with signatures and ignores msi', () => {
  const files = [
    'Kestrel.Mail_0.1.0_x64-setup.exe',
    'Kestrel.Mail_0.1.0_x64-setup.exe.sig',
    'Kestrel.Mail_0.1.0_x64_en-US.msi',
    'Kestrel.Mail_0.1.0_x64_en-US.msi.sig',
    'README.txt',
  ];
  const assets = collectSignedAssets(files, (name) => (name.endsWith('.sig') ? 'sig-data\n' : ''));
  assert.deepEqual(assets, [{ name: 'Kestrel.Mail_0.1.0_x64-setup.exe', signature: 'sig-data' }]);
});

test('fails on a missing or empty signature', () => {
  assert.throws(
    () => collectSignedAssets(['Kestrel.Mail_0.1.0_x64-setup.exe'], () => ''),
    /missing signature/,
  );
  assert.throws(
    () => collectSignedAssets(['Kestrel.Mail_0.1.0_x64-setup.exe', 'Kestrel.Mail_0.1.0_x64-setup.exe.sig'], () => '  '),
    /empty signature/,
  );
});
