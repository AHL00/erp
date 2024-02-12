import { browser } from "$app/environment";


export async function api_call(path: string, method: string, body?: any) {
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