<script lang="ts">
	import type { InventoryItem } from '$bindings/InventoryItem';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';

	// The item that is currently being edited
	let current_item_id: number | null = null;

	let current_editing_item: InventoryItem | null = null;

	let loading_counter = 0;

	export function edit_item(item_id: number) {
		current_item_id = item_id;

		loading_counter++;

		// Fetch the item from the backend
		api_call(`inventory/${item_id}`, 'GET', null).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					current_editing_item = data;
				});
			} else {
				// TODO: Deal with this
				console.log('Error fetching item');
			}
			loading_counter--;
		});
	}

	export function clear_current() {
		current_item_id = null;
		current_editing_item = null;
	}

	export let close_callback: () => void;

	function parse_verify_number_macro(
		n: string,
		display_name: string,
		can_be_negative: boolean = false,
		is_int: boolean = false
	): number | null {
		let number_type_str = is_int ? 'whole number' : 'number';
		try {
			let num = is_int ? parseInt(n) : parseFloat(n);

			if (isNaN(num)) {
				toast.push(`${display_name} must be a ${number_type_str}`);
				return null;
			}

			if (!can_be_negative && num < 0) {
				toast.push(`${display_name} must be a positive ${number_type_str}`);
				return null;
			}

			return num;
		} catch (e) {
			toast.push(`${display_name} must be a ${number_type_str}`);
			return null;
		}
	}

	function submit_form() {
		//TODO: After getting toasts working to show errors
		let name = (document.getElementById('name') as HTMLInputElement).value;

		if (name == '') {
			toast.push('Name cannot be empty');
			return;
		}

		let stock = parse_verify_number_macro(
			(document.getElementById('stock') as HTMLInputElement).value,
			'Stock',
			false
		);
		if (stock == null) return;

		let price = parse_verify_number_macro(
			(document.getElementById('price') as HTMLInputElement).value,
			'Price',
			false
		);
		if (price == null) return;

		let quantity_per_box = parse_verify_number_macro(
			(document.getElementById('quantity_per_box') as HTMLInputElement).value,
			'Quantity per Box',
			false
		);
		if (quantity_per_box == null) return;

		// Submit the form
		let item_data: InventoryItem = {
			id: current_editing_item!.id,
			name: name,
			stock: stock,
			price: price.toString(),
			quantity_per_box: quantity_per_box
		};

		// For when the item is closed before the API call is finished
		let cached_id = current_editing_item!.id;

		api_call(`inventory/${current_editing_item!.id}`, 'PUT', item_data).then((res) => {
			if (res?.status == 200) {
				toast.push('Edited item successfully...', {
					duration: 2000
				});

				edit_callback(cached_id);
			} else {
				toast.push(`Error editing item ${cached_id} (${res!.statusText})`, {
					duration: 5000,
					pausable: true
				});
			}
		});

		end();
	}

	// Fields
	let name_field: HTMLInputElement;
	let stock_field: HTMLInputElement;
	let price_field: HTMLInputElement;
	let quantity_per_box_field: HTMLInputElement;

	/// This is used to update the table in the parent component.
    /// Only gives the id as a last line of defense in case the value on the database
    /// doesn't actually change.
	export let edit_callback: (item_id: number) => void;

	let end = () => {
		close_callback();
		clear_current();
	};
</script>

<div class="flex flex-col h-full w-full items-center">
	<div class="my-4">
		<span class="text-2xl">Edit Item</span>
	</div>

	<form class="flex flex-col space-y-4">
		{#if loading_counter > 0 || current_editing_item == null}
			{#if loading_counter == 0 && current_editing_item == null}
				<div class="flex-grow">Error loading item</div>
			{:else}
				<div class="flex-grow">Loading...</div>
			{/if}
		{:else}
			<!-- Here, the item data should be populated in the variable -->
			<!-- TODO: Reset buttons beside each field -->
			<div class="flex flex-row space-x-2">
				<label for="name">Name</label>
				<input
					class="flex-grow"
					id="name"
					type="text"
					bind:this={name_field}
					value={current_editing_item?.name}
				/>
				<button
					on:click={() =>
						// @ts-ignore
						(name_field.value = current_editing_item?.name)}
				>
					<i class="fa-solid fa-rotate-left"></i>
				</button>
			</div>
			<div class="flex flex-row space-x-2">
				<label for="stock">Stock</label>
				<input
					class="flex-grow"
					type="text"
					id="stock"
					bind:this={stock_field}
					value={current_editing_item?.stock}
				/>
				<button
					on:click={() =>
						// @ts-ignore
						(stock_field.value = current_editing_item?.stock)}
				>
					<i class="fa-solid fa-rotate-left"></i>
				</button>
			</div>
			<div class="flex flex-row space-x-2">
				<label for="price">Price</label>
				<input
					class="flex-grow"
					type="text"
					id="price"
					bind:this={price_field}
					value={current_editing_item?.price}
				/>
				<button
					on:click={() =>
						// @ts-ignore
						(price_field.value = current_editing_item?.price)}
				>
					<i class="fa-solid fa-rotate-left"></i>
				</button>
			</div>
			<div class="flex flex-row space-x-2">
				<label for="quantity_per_box">Quantity per Box</label>
				<input
					class="flex-grow"
					type="text"
					id="quantity_per_box"
					bind:this={quantity_per_box_field}
					value={current_editing_item?.quantity_per_box}
				/>
				<button
					on:click={() =>
						// @ts-ignore
						(quantity_per_box_field.value = current_editing_item?.quantity_per_box)}
				>
					<i class="fa-solid fa-rotate-left"></i>
				</button>
			</div>
		{/if}

		<div class="self-end mt-auto">
			<button on:click={end}>Cancel</button>
			{#if !(loading_counter > 0 || current_editing_item == null)}
				<button type="submit" on:click={submit_form}> Save</button>
			{/if}
		</div>
	</form>
</div>
