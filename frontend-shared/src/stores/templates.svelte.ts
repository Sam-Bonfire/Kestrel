import { getSettings, updateSettings } from '../api/client.js';
import type { Snippet, Signature } from '../api/client.js';

const SNIPPETS_KEY = 'kestrel:settings:snippets';
const SIGNATURES_KEY = 'kestrel:settings:signatures';

function loadJson<T>(key: string, def: T): T {
  try {
    const val = localStorage.getItem(key);
    return val !== null ? JSON.parse(val) : def;
  } catch {
    return def;
  }
}

function saveItem(key: string, val: string): void {
  try {
    localStorage.setItem(key, val);
  } catch {
    // Non-fatal
  }
}

export class TemplateStore {
  snippets = $state<Snippet[]>(loadJson(SNIPPETS_KEY, []));
  signatures = $state<Signature[]>(loadJson(SIGNATURES_KEY, []));

  private isInitializing = false;
  private isUpdating = false;

  async initializeTemplates() {
    if (this.isInitializing) return;
    this.isInitializing = true;
    try {
      const settings = await getSettings();
      if (settings.snippets !== undefined) {
        this.snippets = settings.snippets;
        saveItem(SNIPPETS_KEY, JSON.stringify(this.snippets));
      }
      if (settings.signatures !== undefined) {
        this.signatures = settings.signatures;
        saveItem(SIGNATURES_KEY, JSON.stringify(this.signatures));
      }
    } catch (err) {
      console.error('Failed to load templates from backend', err);
    } finally {
      this.isInitializing = false;
    }
  }

  async syncTemplates() {
    if (this.isInitializing || this.isUpdating) return;
    this.isUpdating = true;

    // Save locally first
    saveItem(SNIPPETS_KEY, JSON.stringify(this.snippets));
    saveItem(SIGNATURES_KEY, JSON.stringify(this.signatures));

    try {
      await updateSettings({
        snippets: this.snippets,
        signatures: this.signatures,
      });
    } catch (err) {
      console.error('Failed to sync templates to backend', err);
    } finally {
      this.isUpdating = false;
    }
  }
}

export const templateStore = new TemplateStore();
