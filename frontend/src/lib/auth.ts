import { writable, type Writable } from 'svelte/store';
import { api_call } from './backend';
import { type AuthInfo } from '$bindings/AuthInfo';
import { goto } from '$app/navigation';

/// This is a store that will hold the login information
/// of the user.
/// If null, the user is not logged in.
/// For type info, check Rust code for the AuthInfo struct.
export const auth_info_store: Writable<AuthInfo | null> = writable(null);

/// Attempt to login
export async function login(username: string, password: string) {
    const response = await api_call('auth/login', 'POST', { username, password });

    if (response === null) {
        console.error('Failed to call login API');
        return;
    }

    if (response?.ok) {
        let refresh_success = await refreshAuthStatus();
        if (!refresh_success) {
            console.error('Failed to refresh auth status after login');
        }
    } else {
        console.error('Login failed');
    }
}

/// Refresh the auth status from the server.
/// This will update the auth_info_store.
/// Returns true if call was successful.
export async function refreshAuthStatus(): Promise<boolean> {
    const response = await api_call('auth/status', 'GET', {});

    if (response === null) {
        console.error('Failed to call refreshAuthStatus API');
        return false;
    }
    
    if (response?.ok) {
        let auth_info: AuthInfo = await response.json();

        auth_info_store.set(auth_info);
        return true;
    } else if (response?.status === 401) {
        // This means the user is not logged in
        auth_info_store.set(null);
        goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
        return true;
    } else {
        console.error('Failed to fetch auth status');
        // Redirect to login page
        goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
        return false;
    }
}

export async function logout() {
    const response = await api_call('auth/logout', 'POST', {});

    if (response === null) {
        console.error('Failed to call logout API');
        return;
    }

    if (response?.ok) {
        auth_info_store.set(null);
    } else {
        console.error('Failed to logout');
    }

    // Should redirect to login page
    window.location.href = '/login';
}