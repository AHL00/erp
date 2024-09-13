<!-- import { get, readable, writable } from 'svelte/store';
import { get_setting } from './backend';
import { onMount } from 'svelte';

let currency_decimal_places_store = writable<number | undefined>(undefined);
let currency_decimal_places_promise = get_setting('currency_decimal_places');

let currency_prefix_store = writable<string | undefined>(undefined);
let currency_prefix_promise = get_setting('currency_prefix');

let currency_suffix_store = writable<string | undefined>(undefined);

// // Run only on the client side
// if (typeof window !== 'undefined') {
// 	// Wait for currency_decimal_promise to resolve
// 	// await currency_decimal_places_promise;
// 	console.log('currency_decimal_places', get(currency_decimal_places_store));
// 	(async () => {
// 		{
// 			let value = await get_setting('currency_prefix');
// 			// @ts-ignore
// 			currency_prefix_store.set(value['String']);
// 		}

// 		{
// 			let value = await get_setting('currency_decimal_places');
// 			// @ts-ignore
// 			currency_decimal_places_store.set(value['Int']);
// 		}
// 	})();
// }

export async function format_currency(amount: number) {
	let negative = amount < 0;
	let abs_amount = Math.abs(amount);

	// Wait for currency_decimal_promise to resolve
	// await currency_decimal_places_promise;
	console.log('currency_decimal_places', get(currency_decimal_places_store));

	let formatted = abs_amount.toFixed(get(currency_decimal_places_store));
	// formatted = formatted.replace('.', currency_decimal_separator!);
	// formatted = formatted.replace(/\B(?=(\d{3})+(?!\d))/g, currency_thousand_separator!);

	// if (currency_prefix) {
	//     formatted = currency_prefix + formatted;
	// }

	// if (currency_suffix) {
	//     formatted = formatted + currency_suffix;
	// }

	// if (negative) {
	//     formatted = '-' + formatted;
	// }

	return formatted;
} -->

<script lang="ts" context="module">
	import { get_setting } from '$lib/backend';
	import Loader from '../Loader.svelte';

	let currency_decimal_places: Promise<number> = (async () => {
		let value = await get_setting('currency_decimal_places');
		// @ts-ignore
		return value['UnsignedInt'];
	})();

	let currency_prefix: Promise<string> = (async () => {
		let value = await get_setting('currency_prefix');
		// @ts-ignore
		return value['Text'];
	})();

	let currency_suffix: Promise<string> = (async () => {
		let value = await get_setting('currency_suffix');
		// @ts-ignore
		return value['Text'];
	})();

	let currency_decimal_separator: Promise<string> = (async () => {
		let value = await get_setting('currency_decimal_separator');
		// @ts-ignore
		return value['Text'];
	})();

	let currency_thousand_separator: Promise<string> = (async () => {
		let value = await get_setting('currency_thousand_separator');
		// @ts-ignore
		return value['Text'];
	})();

	async function format(value: number) {
		let negative = value < 0;
		let abs_value = Math.abs(value);

		let formatted = abs_value.toFixed(await currency_decimal_places);
		formatted = formatted.replace('.', await currency_decimal_separator);
		formatted = formatted.replace(/\B(?=(\d{3})+(?!\d))/g, await currency_thousand_separator);

		if (await currency_prefix) {
			formatted = (await currency_prefix) + formatted;
		}

		if (await currency_suffix) {
			formatted = formatted + (await currency_suffix);
		}

		if (negative) {
			formatted = '-' + formatted;
		}

		return formatted;
	}
</script>

<script lang="ts">
	export let value: number;
	let formatted_value: Promise<string>;
	$: {
		formatted_value = format(value);
	}
</script>

<span>
	{#await formatted_value}
		<Loader icon_size={0.5} />
	{:then formatted}
		<span>{formatted}</span>
	{:catch error}
		<span class="text-red-500">[Formatting Error]</span>
	{/await}
</span>
