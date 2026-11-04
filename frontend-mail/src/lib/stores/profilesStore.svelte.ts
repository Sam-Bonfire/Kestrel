export type Profile = {
  id: string;
  name: string;
  accountIds: string[]; // 'all' means all accounts, or array of specific account ids
};

const STORAGE_KEY = 'kestrel:mail:profiles';

export const profilesStore = $state({
  profiles: [] as Profile[],
  activeProfileId: 'default' as string,
  loaded: false,

  init() {
    if (typeof localStorage !== 'undefined') {
      const savedStr = localStorage.getItem(STORAGE_KEY);
      if (savedStr) {
        try {
          const saved = JSON.parse(savedStr);
          if (saved && saved.profiles && saved.profiles.length > 0) {
            this.profiles = saved.profiles;
            this.activeProfileId = saved.activeProfileId || 'default';
          }
        } catch (e) {
          console.error('Failed to parse profiles from localStorage', e);
        }
      }
    }

    if (this.profiles.length === 0) {
      this.profiles = [{ id: 'default', name: 'All Accounts', accountIds: ['all'] }];
      this.activeProfileId = 'default';
    }
    this.loaded = true;
  },

  persist() {
    if (!this.loaded) return;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        profiles: $state.snapshot(this.profiles),
        activeProfileId: this.activeProfileId
      }));
    }
  },

  addProfile(name: string, accountIds: string[]) {
    const newProfile = { id: crypto.randomUUID(), name, accountIds };
    this.profiles.push(newProfile);
    this.activeProfileId = newProfile.id;
    this.persist();
  },

  updateProfile(id: string, name: string, accountIds: string[]) {
    const profile = this.profiles.find(p => p.id === id);
    if (profile) {
      profile.name = name;
      profile.accountIds = accountIds;
      this.persist();
    }
  },

  deleteProfile(id: string) {
    if (id === 'default') return;
    this.profiles = this.profiles.filter(p => p.id !== id);
    if (this.activeProfileId === id) {
      this.activeProfileId = 'default';
    }
    this.persist();
  },

  setActiveProfile(id: string) {
    if (this.profiles.some(p => p.id === id)) {
      this.activeProfileId = id;
      this.persist();
    }
  },

  getActiveProfile() {
    return this.profiles.find(p => p.id === this.activeProfileId) || this.profiles[0];
  }
});
