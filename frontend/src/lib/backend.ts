import { browser } from "$app/environment";

export const api_base = import.meta.env.VITE_API_BASE_URL ? import.meta.env.VITE_API_BASE_URL : window.location.origin + '/api';

export async function api_call(path: string, method: string, body: any) {
    if (!browser) return;

    if (method === 'GET') {
        return fetch(`${api_base}/${path}`, {
            method: method,
            headers: {
                Accept: "application/json",
            },
            credentials: "include",
        });
    }

    if (method === 'HEAD') {
        return fetch(`${api_base}/${path}`, {
            method: method,
            headers: {
                Accept: "application/json",
            },
            credentials: "include",
        });
    }

    return fetch(`${api_base}/${path}`, {
        method: method,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        credentials: "include",
        body: JSON.stringify(body),
    });
}