// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { OrderItemPostRequest } from "./OrderItemPostRequest";

export type OrderPostRequest = { customer_id: number, retail: boolean, notes: string, amount_paid: string, items: Array<OrderItemPostRequest>, };