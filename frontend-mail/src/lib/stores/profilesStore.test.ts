import { describe, it, expect, beforeEach, vi } from 'vitest';
import { profilesStore } from './profilesStore.svelte.js';

describe('profilesStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    profilesStore.profiles = [];
    profilesStore.activeProfileId = 'default';
    profilesStore.loaded = false;
    localStorage.clear();
  });

  it('initializes with default profile if no saved profiles', () => {
    profilesStore.init();
    expect(profilesStore.profiles.length).toBe(1);
    expect(profilesStore.profiles[0].id).toBe('default');
    expect(profilesStore.activeProfileId).toBe('default');
    expect(profilesStore.loaded).toBe(true);
  });

  it('initializes with saved profiles', () => {
    localStorage.setItem('kestrel:mail:profiles', JSON.stringify({
      profiles: [{ id: '1', name: 'Work', accountIds: ['a1'] }],
      activeProfileId: '1'
    }));
    profilesStore.init();
    expect(profilesStore.profiles.length).toBe(1);
    expect(profilesStore.activeProfileId).toBe('1');
  });

  it('adds a new profile', () => {
    profilesStore.loaded = true;
    profilesStore.addProfile('Personal', ['a2']);
    expect(profilesStore.profiles.length).toBe(1);
    expect(profilesStore.profiles[0].name).toBe('Personal');
    expect(profilesStore.activeProfileId).toBe(profilesStore.profiles[0].id);
  });
});
