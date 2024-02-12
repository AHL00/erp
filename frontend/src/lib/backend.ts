import { browser } from "$app/environment";

// ENV
export let api_base = import.meta.env.VITE_API_BASE? import.meta.env.VITE_API_BASE : `${window.location.origin}/api`;

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
        });
    }

    if (method === 'HEAD') {
        return fetch(`${api_base}/${path}`, {
            method: method,
        });
    }

    return fetch(`${api_base}/${path}`, {
        method: method,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
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