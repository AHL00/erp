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
    export let custom_class: string = "";

	let formatted_value: Promise<string>;
	$: {
		formatted_value = format(value);
	}
</script>

<span>
	{#await formatted_value}
		<Loader icon_size={0.5} />
	{:then formatted}
		<span class={custom_class}>{formatted}</span>
	{:catch error}
		<span class="text-red-500">[Formatting Error]</span>
	{/await}
</span>
