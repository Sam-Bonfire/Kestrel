import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const isPermissionGranted = vi.fn();
const requestPermission = vi.fn();
const sendNotification = vi.fn();
const registerActionTypes = vi.fn();

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted,
  requestPermission,
  sendNotification,
  registerActionTypes,
}));

const {
  ensureNotificationPermission,
  sendDesktopNotification,
  registerNotificationCategories,
} = await import('./notifications.js');

function enterTauri() {
  (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {};
}

function leaveTauri() {
  delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
}

describe('desktop notifications', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    enterTauri();
  });

  afterEach(() => {
    leaveTauri();
  });

  it('does not touch the OS outside Tauri', async () => {
    leaveTauri();
    expect(await ensureNotificationPermission()).toBe(false);
    expect(await sendDesktopNotification({ title: 'Hi' })).toBe(false);
    await registerNotificationCategories([{ id: 'cat', actions: [] }]);
    expect(isPermissionGranted).not.toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
    expect(registerActionTypes).not.toHaveBeenCalled();
  });

  it('reuses an existing grant without prompting', async () => {
    isPermissionGranted.mockResolvedValue(true);
    expect(await ensureNotificationPermission()).toBe(true);
    expect(requestPermission).not.toHaveBeenCalled();
  });

  it('prompts once when permission was never decided', async () => {
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('granted');
    expect(await ensureNotificationPermission()).toBe(true);
  });

  it('does not send when permission is denied', async () => {
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('denied');
    expect(await sendDesktopNotification({ title: 'Nope' })).toBe(false);
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('sends with the action category once granted', async () => {
    isPermissionGranted.mockResolvedValue(true);
    const notification = { id: 7, title: 'New mail', body: 'Hello', actionTypeId: 'new_email_actions' };
    expect(await sendDesktopNotification(notification)).toBe(true);
    expect(sendNotification).toHaveBeenCalledWith(notification);
  });

  it('registers action categories in Tauri', async () => {
    const categories = [{ id: 'event_actions', actions: [{ id: 'snooze', title: 'Snooze' }] }];
    await registerNotificationCategories(categories);
    expect(registerActionTypes).toHaveBeenCalledWith(categories);
  });
});
