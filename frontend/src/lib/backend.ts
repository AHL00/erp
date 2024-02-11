import { browser } from "$app/environment";


export async function api_call(path: string, method: string, body: any) {
    if (!browser) {
        console.error('Tried to make API call from server');
        return null;
    }

    if (method === 'GET') {
        return fetch(`${window.location.origin}/api/${path}`, {
            method: method,
            headers: {
                Accept: "application/json",
            },
        });
    }

    if (method === 'HEAD') {
        return fetch(`${window.location.origin}/api/${path}`, {
            method: method,
        });
    }

    return fetch(`${window.location.origin}/api/${path}`, {
        method: method,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    });
}