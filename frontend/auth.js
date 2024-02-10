import { writable } from 'svelte/store';

/// This is a store that will hold the login information
/// of the user.
/// If null, the user is not logged in.
/// For type info, check Rust code for the AuthInfo struct.
export const auth_info = writable(null);

export async function login(username, password) {
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
    });

    if (response.ok) {
        refreshAuthInfo();
    } else {
        console.error('Login failed');
    }
}

export async function refreshAuthInfo() {
    const response = await fetch('/api/auth/info', {
        method: 'GET',
    });
    
    if (response.ok) {
        auth_info.set(await response.json());
    } else {
        console.error('Failed to fetch auth info');
    }
}

export async function logout() {
    const response = await fetch('/api/auth/logout', {
        method: 'POST',
    });

    if (response.ok) {
        auth_info.set(null);
    } else {
        console.error('Failed to logout');
    }
}