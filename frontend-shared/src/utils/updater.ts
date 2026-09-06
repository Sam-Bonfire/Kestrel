export type UpdateStatus =
  | { state: 'up-to-date' }
  | { state: 'available'; version: string }
  | { state: 'unavailable'; reason: string };

function isTauri(): boolean {
  return typeof window !== 'undefined' && (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== undefined;
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
    return { state: 'unavailable', reason: err instanceof Error ? err.message : String(err) };
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
    return err instanceof Error ? err.message : String(err);
  }
}
