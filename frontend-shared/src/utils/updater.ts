export type UpdateStatus =
  | { state: 'up-to-date' }
  | { state: 'available'; version: string }
  | { state: 'unavailable'; reason: string };

function isTauri(): boolean {
  return typeof window !== 'undefined' && (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== undefined;
}

/** Translate raw updater failures into actionable copy (K-1467).
 * The Tauri updater fetches a JSON manifest published with the release;
 * when it is missing the plugin throws a serde/JSON error that meant
 * nothing to users. Never surface that raw text. */
export function describeUpdateError(err: unknown): string {
  if (err == null) return 'Update check failed. Please try again later.';
  const msg = err instanceof Error ? err.message : String(err);
  if (/json/i.test(msg)) {
    return 'No update manifest published for this app yet — updates activate once a release ships one. Check back after the next release.';
  }
  if (/404|not found/i.test(msg)) {
    return 'No update manifest published for this app yet. Check back after the next release.';
  }
  return msg || 'Update check failed. Please try again later.';
}

/** Check the release feed for a newer desktop build. Never throws. */
export async function checkForAppUpdate(): Promise<UpdateStatus> {
  if (!isTauri()) return { state: 'unavailable', reason: 'Not running in the desktop app' };
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();
    if (!update) return { state: 'up-to-date' };
    return { state: 'available', version: update.version };
  } catch (err) {
    return { state: 'unavailable', reason: describeUpdateError(err) };
  }
}

/** Download and install an available update. Returns a message for UI display. */
export async function installAppUpdate(): Promise<string> {
  if (!isTauri()) return 'Updates are only available in the desktop app';
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();
    if (!update) return 'Already up to date';
    await update.downloadAndInstall();
    return 'Update installed, please restart the app to apply it';
  } catch (err) {
    return describeUpdateError(err);
  }
}
