import { writable } from 'svelte/store';
import { api_base } from './backend';

/// This is a store that will hold the login information
/// of the user.
/// If null, the user is not logged in.
/// For type info, check Rust code for the AuthInfo struct.
export const auth_info_store: any = writable(null);

/// Attempt to login
export async function login(username: string, password: string) {
    const response = await fetch(`${api_base}auth/login`, {
        method: 'POST',
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
    });

    if (response.ok) {
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
    const response = await fetch(`${api_base}auth/status`, {
        method: 'GET',
    });
    
    if (response.ok) {
        auth_info_store.set(await response.json());
        return true;
    } else if (response.status === 401) {
        auth_info_store.set(null);
        return true;
    } else {
        console.error('Failed to fetch auth status');
        return false;
    }
}

export async function logout() {
    const response = await fetch(`${api_base}auth/logout`, {
        method: 'POST',
        credentials: 'include'
    });

    if (response.ok) {
        auth_info_store.set(null);
    } else {
        console.error('Failed to logout');
    }

    // Should redirect to login page
    window.location.href = '/login';
}