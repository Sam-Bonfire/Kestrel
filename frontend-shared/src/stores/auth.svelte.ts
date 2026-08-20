import { getMe } from '../api/client.js';

export const authState = $state<{
    userId: string | null;
    isInitialized: boolean;
    isAuthenticated: boolean;
}>({
    userId: null,
    isInitialized: false,
    get isAuthenticated() {
        return !!this.userId;
    }
});

export async function initAuth() {
    try {
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
}

export const revokedAccounts = $state<{accountId: string, provider: string}[]>([]);

export function addRevokedAccount(accountId: string, provider: string) {
    if (!revokedAccounts.find(a => a.accountId === accountId)) {
        revokedAccounts.push({ accountId, provider });
    }
}
