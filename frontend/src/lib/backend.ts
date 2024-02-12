import { browser } from "$app/environment";

// ENV
export let api_base = import.meta.env.VITE_API_BASE;

if (!api_base) {
    console.error('API_BASE not set in environment');
}

export async function api_call(path: string, method: string, body?: any) {
    if (!browser) {
        console.error('Tried to make API call from server');
        return null;
    }

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

export class Product {
    id: number;
    name: string;
    price: number;
    stock: number;
    quantity_per_carton: number;

    constructor(id: number, name: string, price: number, stock: number, quantity_per_carton: number) {
        this.id = id;
        this.name = name;
        this.price = price;
        this.stock = stock;
        this.quantity_per_carton = quantity_per_carton;
    }
}