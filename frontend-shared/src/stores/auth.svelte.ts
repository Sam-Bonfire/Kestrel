import { getMe } from '../api/client.js';
import { invoke } from '@tauri-apps/api/core';

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

export async function initAuth() {
    try {
        if ((window as any).__TAURI_INTERNALS__) {
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
    } catch {
        authState.userId = null;
    } finally {
        authState.isInitialized = true;
    }
}

export async function login(username: string, password: string) {
    try {
        const res = await fetch('http://localhost:8080/api/v1/auth/token', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
            body: JSON.stringify({ username, password }),
        });

        if (!res.ok) {
            throw new Error('Login failed');
        }

        const data = await res.json();
        
        authState.userId = data.user_id;
        if (data.token) {
            authState.token = data.token;
            if ((window as any).__TAURI_INTERNALS__) {
                try {
                    await invoke('set_keychain_token', { token: data.token });
                } catch (e) {
                    console.error("Failed to set keychain token", e);
                }
            }
        }
        authState.isInitialized = true;
        
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
    if ((window as any).__TAURI_INTERNALS__) {
        invoke('delete_keychain_token').catch(e => {
            console.error("Failed to delete keychain token", e);
        });
    }
}
