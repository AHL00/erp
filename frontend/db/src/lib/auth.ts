import { get, writable, type Writable } from 'svelte/store';
import { api_call } from './backend';
import { type User } from '$bindings/User';
import { goto } from '$app/navigation';
import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';

/// This is a store that will hold the login information
/// of the user.
/// If null, the user is not logged in.
/// For type info, check Rust code for the AuthInfo struct.
export const auth_info_store: Writable<User | null> = writable(null);

export function user_has_permission(permission: UserPermissionEnum): boolean {
	const auth_info = get(auth_info_store);

	if (auth_info === null) {
		return false;
	}

	return auth_info.permissions.includes(permission) || auth_info.permissions.includes('ADMIN');
}

export enum LoginStatus {
	INCORRECT_CREDENTIALS,
	FAILED_TO_REACH_SERVER,
    FAILED_TO_REFRESH_STATUS,
    SERVER_ERROR,
    SUCCESS
}

/// Attempt to login
export async function login(username: string, password: string) {
	const response = await api_call('auth/login', 'POST', { username, password });

	if (response === null) {
		return LoginStatus.FAILED_TO_REACH_SERVER;
	}

	if (response?.ok) {
		let refresh_success = await refreshAuthStatus();
		if (!refresh_success) {
            return LoginStatus.FAILED_TO_REFRESH_STATUS;
		}
        return LoginStatus.SUCCESS
	} else if (response?.status === 401) {
        return LoginStatus.INCORRECT_CREDENTIALS;
    }
    else if (response?.status === 500) {
        return LoginStatus.SERVER_ERROR;
    }
    else {
        return LoginStatus.FAILED_TO_REACH_SERVER;
    }
}

export enum auth_status {
	AUTHENTICATED,
	FETCH_ERROR,
	NOT_AUTHENTICATED,
	LOADING
}

export const auth_status_store: Writable<auth_status> = writable(auth_status.NOT_AUTHENTICATED);

/// Refresh the auth status from the server.
/// This will update the auth_info_store.
/// Returns true if authenticated, false if not.
/// Rejected promise if failed to call the API.
/// TODO: refresh automatically constantly, maybe every 5 seconds?
export async function refreshAuthStatus(): Promise<boolean> {
	auth_status_store.set(auth_status.LOADING);

	return new Promise((resolve, reject) => {
		api_call('auth/status', 'GET', {})
			.then((response) => {
				if (response?.ok) {
					response.json().then((data) => {
						auth_info_store.set(data);
						auth_status_store.set(auth_status.AUTHENTICATED);
						resolve(true);
					});
				} else if (response?.status === 401) {
					// This means the user is not logged in
					auth_info_store.set(null);
					auth_status_store.set(auth_status.NOT_AUTHENTICATED);
					goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
					resolve(false);
				} else {
					// If there's an error, it's probably safe to log out. This still leaves the cookie.
					auth_info_store.set(null);
					auth_status_store.set(auth_status.FETCH_ERROR);
					console.error(
						'Failed to fetch auth status for unknown reason: HTTP code ' + response?.status
					);
					// Redirect to login page
					goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
					resolve(false);
				}
			})
			.catch((error) => {
				// If failed to call, revoke the auth status
				auth_info_store.set(null);
				auth_status_store.set(auth_status.FETCH_ERROR);
				console.error('Failed to call refreshAuthStatus API');
				reject();
			});
	});
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
