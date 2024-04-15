<script lang="ts">
	import type { InventoryItem } from '$bindings/InventoryItem';
	import type { InventoryItemPatchRequest } from '$bindings/InventoryItemPatchRequest';
	import { InventoryField } from './field';

	export let item: InventoryItem;

	function update_field(field: InventoryField, value: any) {
		let request: InventoryItemPatchRequest = {
			name: null,
			price: null,
			stock: null,
			quantity_per_box: null
		};

		switch (field) {
			case InventoryField.Name:
				request.name = value;
				break;
			case InventoryField.Price:
				request.price = value;
				break;
			case InventoryField.Stock:
				request.stock = value;
				break;
			case InventoryField.QuantityPerBox:
				request.quantity_per_box = value;
				break;
		}
	}

    function reset_item() {
        item = {
            id: item.id,
            name: item.name,
            price: item.price,
            stock: item.stock,
            quantity_per_box: item.quantity_per_box
        };
    }
</script>

<tr class="text-center items-center">
	<td class="p-2">{item.id}</td>
	<td class="p-2">{item.name}</td>
	<td class="p-2">{item.price}</td>
	<td class="p-2">{item.stock}</td>

	<td class="p-2 items-center inline-flex">
		<div
			class="relative flex items-center max-w-[8rem] outline outline-1 rounded-lg
        outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline
        "
		>
			<button
				type="button"
				class="text-sm text-custom-text-light-lighter dark:text-custom-text-dark-lighter
                outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline
                bg-custom-bg-lighter dark:bg-custom-bg-dark
                rounded-l-lg px-2 h-8 z-10"
				on:click={() => {
					// @ts-ignore
					let el = document.getElementById(`qpb-input-${item.id}`);
					// @ts-ignore
					el.value = Math.max(0, el.value - 1);
				}}
			>
				<i class="fa-solid fa-minus"></i>
			</button>
			<input
				type="text"
				id="qpb-input-{item.id}"
				class="border-x-0 h-8 text-center text-sm block w-full bg-custom-bg-lighter dark:bg-custom-bg-dark outline-none border-none"
				value={item.quantity_per_box}
				required
			/>
			<button
				type="button"
				class="text-sm text-custom-text-light-lighter dark:text-custom-text-dark-lighter
                outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline
                bg-custom-bg-lighter dark:bg-custom-bg-dark
                rounded-e-lg px-2 h-8 z-10"
				on:click={() => {
					// @ts-ignore
					let el = document.getElementById(`qpb-input-${item.id}`);
					// @ts-ignore
					el.value = parseInt(el.value) + 1;
				}}
			>
				<i class="fa-solid fa-plus"></i>
			</button>
		</div>
	</td>

	<td class="p-2 items-center">
		<button
			on:click={() => {
				// @ts-ignore

			}}
		>
			<i class="fa-solid fa-rotate-left"></i>
		</button>
	</td>
</tr>

<style>
</style>
