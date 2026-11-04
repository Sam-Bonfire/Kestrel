export type TabState = {
  id: string;
  view: string;
  searchQuery?: string;
  threadId?: string | null;
  activeCategoryId?: string;
  activeLabelFilter?: string;
};

const STORAGE_KEY = 'kestrel:mail:tabs';

export const tabsStore = $state({
  tabs: [] as TabState[],
  activeTabId: null as string | null,
  closedTabs: [] as TabState[],
  loaded: false,

  init() {
    if (typeof localStorage !== 'undefined') {
      const savedStr = localStorage.getItem(STORAGE_KEY);
      if (savedStr) {
        try {
          const saved = JSON.parse(savedStr);
          if (saved && saved.tabs && saved.tabs.length > 0) {
            this.tabs = saved.tabs;
            this.activeTabId = saved.activeTabId || saved.tabs[0].id;
            this.closedTabs = saved.closedTabs || [];
          }
        } catch (e) {
          console.error('Failed to parse tabs from localStorage', e);
        }
      }
    }

    if (this.tabs.length === 0) {
      const defaultTab = { id: crypto.randomUUID(), view: 'inbox' };
      this.tabs = [defaultTab];
      this.activeTabId = defaultTab.id;
    }
    this.loaded = true;
  },

  persist() {
    if (!this.loaded) return;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        tabs: $state.snapshot(this.tabs),
        activeTabId: this.activeTabId,
        closedTabs: $state.snapshot(this.closedTabs).slice(-10) // keep last 10
      }));
    }
  },

  openTab(tabData: Partial<TabState>, inBackground = false) {
    const newTab: TabState = {
      id: crypto.randomUUID(),
      view: tabData.view || 'inbox',
      searchQuery: tabData.searchQuery,
      threadId: tabData.threadId,
      ...tabData
    };
    this.tabs.push(newTab);
    if (!inBackground) {
      this.activeTabId = newTab.id;
    }
    this.persist();
  },

  closeTab(id: string) {
    const index = this.tabs.findIndex(t => t.id === id);
    if (index === -1) return;

    const tab = this.tabs[index];
    this.tabs.splice(index, 1);
    this.closedTabs.push(tab);

    if (this.tabs.length === 0) {
      const defaultTab = { id: crypto.randomUUID(), view: 'inbox' };
      this.tabs.push(defaultTab);
      this.activeTabId = defaultTab.id;
    } else if (this.activeTabId === id) {
      const nextIndex = Math.min(index, this.tabs.length - 1);
      this.activeTabId = this.tabs[nextIndex].id;
    }
    this.persist();
  },

  restoreTab() {
    const tab = this.closedTabs.pop();
    if (tab) {
      this.tabs.push(tab);
      this.activeTabId = tab.id;
      this.persist();
    }
  },

  switchTab(id: string) {
    if (this.tabs.some(t => t.id === id)) {
      this.activeTabId = id;
      this.persist();
    }
  },

  cycleNext() {
    if (this.tabs.length <= 1) return;
    const index = this.tabs.findIndex(t => t.id === this.activeTabId);
    const nextIndex = (index + 1) % this.tabs.length;
    this.activeTabId = this.tabs[nextIndex].id;
    this.persist();
  },

  cyclePrev() {
    if (this.tabs.length <= 1) return;
    const index = this.tabs.findIndex(t => t.id === this.activeTabId);
    const prevIndex = (index - 1 + this.tabs.length) % this.tabs.length;
    this.activeTabId = this.tabs[prevIndex].id;
    this.persist();
  },

  updateActiveTab(update: Partial<TabState>) {
    if (!this.activeTabId) return;
    const tab = this.tabs.find(t => t.id === this.activeTabId);
    if (tab) {
      Object.assign(tab, update);
      this.persist();
    }
  }
});
