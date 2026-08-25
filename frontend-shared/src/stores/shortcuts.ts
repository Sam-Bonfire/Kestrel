import { tinykeys } from 'tinykeys';
import { writable, get } from 'svelte/store';

// Default keyboard shortcuts map
export const defaultShortcuts = [
  { id: 'compose', label: 'Compose New Message', defaultKey: 'C' },
  { id: 'command', label: 'Command Palette', defaultKey: '$mod+K' },
  { id: 'day_view', label: 'Day View (Calendar)', defaultKey: 'D' },
  { id: 'week_view', label: 'Week View (Calendar)', defaultKey: 'W' },
  { id: 'month_view', label: 'Month View (Calendar)', defaultKey: 'M' },
  { id: 'agenda_view', label: 'Agenda View (Calendar)', defaultKey: 'A' },
  { id: 'inbox', label: 'Go to Inbox', defaultKey: 'I' },
  { id: 'archive', label: 'Archive Message', defaultKey: 'E' },
  { id: 'reply', label: 'Reply', defaultKey: 'R' },
  { id: 'today', label: 'Go to Today', defaultKey: 'T' },
  { id: 'escape', label: 'Escape', defaultKey: 'Escape' },
];

export type ShortcutOverride = Record<string, string>;

const SHORTCUTS_KEY = 'kestrel:settings:shortcuts';

function loadOverrides(): ShortcutOverride {
  try {
    const val = localStorage.getItem(SHORTCUTS_KEY);
    return val !== null ? JSON.parse(val) : {};
  } catch {
    return {};
  }
}

export const customShortcuts = writable<ShortcutOverride>(loadOverrides());

customShortcuts.subscribe((val) => {
  try {
    localStorage.setItem(SHORTCUTS_KEY, JSON.stringify(val));
  } catch {
    // Non-fatal
  }
});

export function resetShortcuts() {
  customShortcuts.set({});
}

export function updateShortcut(id: string, keyCombo: string) {
  customShortcuts.update(overrides => {
    return { ...overrides, [id]: keyCombo };
  });
}

/**
 * Checks if the user is currently typing in an input field.
 * This should be used to guard global keyboard shortcuts.
 */
export function inputGuard(event: KeyboardEvent): boolean {
  const target = event.target as HTMLElement;
  if (!target) return false;

  // Modifiers (Cmd/Ctrl+K, Cmd/Ctrl+Enter, Esc) always bypass the guard.
  if (event.key === 'Escape' || event.ctrlKey || event.metaKey) {
    return false;
  }

  return (
    target.tagName === 'INPUT' ||
    target.tagName === 'TEXTAREA' ||
    target.tagName === 'SELECT' ||
    target.isContentEditable
  );
}

/**
 * Registers global keyboard shortcuts with tinykeys, applying the input guard
 * and resolving any user-defined overrides.
 *
 * @param node The element to attach the listener to (usually window)
 * @param actionMap A map of shortcut IDs to callback functions
 * @returns An unsubscribe function
 */
export function registerShortcuts(node: HTMLElement | Window, actionMap: Record<string, (e: KeyboardEvent) => void>) {
  let unsubscribeTinykeys: () => void;

  const unsubscribeStore = customShortcuts.subscribe(overrides => {
    if (unsubscribeTinykeys) {
      unsubscribeTinykeys();
    }

    const keymap: Record<string, (e: KeyboardEvent) => void> = {};

    for (const [id, callback] of Object.entries(actionMap)) {
      const def = defaultShortcuts.find(s => s.id === id);
      if (def) {
        let keyCombo = overrides[id] || def.defaultKey;
        // tinykeys uses $mod for Cmd/Ctrl

        // Wrap the callback in the input guard
        keymap[keyCombo] = (e: KeyboardEvent) => {
          if (!inputGuard(e)) {
            callback(e);
          }
        };
      }
    }

    unsubscribeTinykeys = tinykeys(node, keymap, {
       // Only block on keydown by default
       event: 'keydown',
       // capture allows shortcuts to trigger early, useful for modals
       capture: false
    });
  });

  return () => {
    unsubscribeStore();
    if (unsubscribeTinykeys) {
      unsubscribeTinykeys();
    }
  };
}
