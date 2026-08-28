import { getMe, createToken } from '../api/client.js';
import { invoke } from '@tauri-apps/api/core';

declare global {
    interface Window {
        __TAURI_INTERNALS__?: Record<string, unknown>;
    }
}

export const authState = $state<{
    userId: string | null;
    token: string | null;
    isInitialized: boolean;
    isAuthenticated: boolean;
}>({
    userId: null,
    token: null,
    isInitialized: false,
    get isAuthenticated() {
        return !!this.userId;
    }
});

import { initializeSettings } from "./settings.js";

export async function initAuth() {
    try {
        if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
            try {
                const keychainToken = await invoke<string>('get_keychain_token');
                if (keychainToken) {
                    authState.token = keychainToken;
                }
            } catch (e) {
                console.error("Failed to get keychain token", e);
            }
        }
        const { user_id } = await getMe();
        authState.userId = user_id;
        await initializeSettings();
    } catch {
        authState.userId = null;
    } finally {
        authState.isInitialized = true;
    }
}

export async function login(username: string, password: string) {
    try {
        const data = await createToken(username, password);
        
        authState.userId = data.user_id;
        if (data.token) {
            authState.token = data.token;
            if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
                try {
                    await invoke('set_keychain_token', { token: data.token });
                } catch (e) {
                    console.error("Failed to set keychain token", e);
                }
            }
        }
        authState.isInitialized = true;
        await initializeSettings();
        
        return { success: true };
    } catch (e) {
        return { success: false, error: e instanceof Error ? e.message : 'Unknown error' };
    }
}

export function logout() {
    // For cookies, we might need a /auth/logout endpoint to clear it, 
    // but clearing state ensures the app drops them
    authState.userId = null;
    authState.token = null;
    if (window.__TAURI_INTERNALS__) {
        invoke('delete_keychain_token').catch(e => {
            console.error("Failed to delete keychain token", e);
        });
    }
}

export const revokedAccounts = $state<{accountId: string, provider: string}[]>([]);

export function addRevokedAccount(accountId: string, provider: string) {
    if (!revokedAccounts.find(a => a.accountId === accountId)) {
        revokedAccounts.push({ accountId, provider });
    }
}
