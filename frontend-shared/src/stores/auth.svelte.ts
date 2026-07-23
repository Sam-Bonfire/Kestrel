export const authState = $state({
    token: localStorage.getItem('kestrel_token'),
    userId: localStorage.getItem('kestrel_user_id'),
    get isAuthenticated() {
        return !!this.token;
    }
});

export async function login(username: string, password: string) {
    try {
        const res = await fetch('http://127.0.0.1:8080/api/v1/auth/token', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password }),
        });

        if (!res.ok) {
            throw new Error('Login failed');
        }

        const data = await res.json();
        
        localStorage.setItem('kestrel_token', data.token);
        localStorage.setItem('kestrel_user_id', data.user_id);
        
        authState.token = data.token;
        authState.userId = data.user_id;
        
        return { success: true };
    } catch (e) {
        return { success: false, error: e instanceof Error ? e.message : 'Unknown error' };
    }
}

export function logout() {
    localStorage.removeItem('kestrel_token');
    localStorage.removeItem('kestrel_user_id');
    authState.token = null;
    authState.userId = null;
}
