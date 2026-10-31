import { describe, it, expect, beforeEach, vi } from 'vitest';
import { tabsStore } from './tabsStore.svelte.js';

describe('tabsStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    tabsStore.tabs = [];
    tabsStore.activeTabId = null;
    tabsStore.closedTabs = [];
    tabsStore.loaded = false;
    localStorage.clear();
  });

  it('initializes with default tab if no saved tabs', () => {
    tabsStore.init();
    expect(tabsStore.tabs.length).toBe(1);
    expect(tabsStore.tabs[0].view).toBe('inbox');
    expect(tabsStore.activeTabId).toBe(tabsStore.tabs[0].id);
    expect(tabsStore.loaded).toBe(true);
  });

  it('initializes with saved tabs', () => {
    localStorage.setItem('kestrel:mail:tabs', JSON.stringify({
      tabs: [{ id: '1', view: 'unread' }],
      activeTabId: '1',
      closedTabs: []
    }));
    tabsStore.init();
    expect(tabsStore.tabs.length).toBe(1);
    expect(tabsStore.tabs[0].view).toBe('unread');
    expect(tabsStore.activeTabId).toBe('1');
  });

  it('opens a new tab and sets it active', () => {
    tabsStore.loaded = true;
    tabsStore.openTab({ view: 'all-mail' });
    expect(tabsStore.tabs.length).toBe(1);
    expect(tabsStore.tabs[0].view).toBe('all-mail');
    expect(tabsStore.activeTabId).toBe(tabsStore.tabs[0].id);
  });

  it('opens a new tab in background', () => {
    tabsStore.loaded = true;
    tabsStore.tabs = [{ id: '1', view: 'inbox' }];
    tabsStore.activeTabId = '1';

    tabsStore.openTab({ view: 'search', searchQuery: 'test' }, true);

    expect(tabsStore.tabs.length).toBe(2);
    expect(tabsStore.activeTabId).toBe('1'); // Should not change
  });

  it('closes a tab and remembers it', () => {
    tabsStore.loaded = true;
    tabsStore.tabs = [
      { id: '1', view: 'inbox' },
      { id: '2', view: 'all-mail' }
    ];
    tabsStore.activeTabId = '1';

    tabsStore.closeTab('1');
    expect(tabsStore.tabs.length).toBe(1);
    expect(tabsStore.closedTabs.length).toBe(1);
    expect(tabsStore.closedTabs[0].id).toBe('1');
    expect(tabsStore.activeTabId).toBe('2');
  });

  it('restores closed tab', () => {
    tabsStore.loaded = true;
    tabsStore.closedTabs = [{ id: '1', view: 'search' }];
    tabsStore.tabs = [{ id: '2', view: 'inbox' }];
    tabsStore.activeTabId = '2';

    tabsStore.restoreTab();
    expect(tabsStore.tabs.length).toBe(2);
    expect(tabsStore.tabs[1].id).toBe('1');
    expect(tabsStore.activeTabId).toBe('1');
  });
});
