import { browser } from '$app/environment';
import type { SettingValue } from '$bindings/SettingValue';

export const api_base = import.meta.env.VITE_API_BASE_URL
	? import.meta.env.VITE_API_BASE_URL
	: window.location.origin + '/api';

type Method = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'HEAD' | 'PATCH';

export async function api_call(path: string, method: Method, body: any) {
	if (!browser) return;

	if (method === 'GET') {
		return fetch(`${api_base}/${path}`, {
			method: method,
			headers: {
				Accept: 'application/json'
			},
			credentials: 'include'
		});
	}

	if (method === 'HEAD') {
		return fetch(`${api_base}/${path}`, {
			method: method,
			headers: {
				Accept: 'application/json'
			},
			credentials: 'include'
		});
	}

	return fetch(`${api_base}/${path}`, {
		method: method,
		headers: {
			Accept: 'application/json',
			'Content-Type': 'application/json'
		},
		credentials: 'include',
		body: JSON.stringify(body)
	});
}

export async function get_setting(key: string) {
	try {
		const response = await api_call(`settings/get_one/${key}`, 'GET', null);
	
        if (!response) {
            throw new Error(`No response from server`);
        }

        if (response.status === 200) {
            let data = await response.json();
            let settings_value: SettingValue = data.value;
            return settings_value;
        } else {
            throw new Error(`HTTP status ${response.status}`);
        }
    } catch (e) {
		throw new Error(`Failed to get setting ${key}: ${e}`);
	}
}
