<script lang="ts">
	import { type InventoryItemListRequest } from '$bindings/InventoryItemListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import { api_call } from '$lib/backend';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';

	let current_item_list_req: InventoryItemListRequest = {
		range: {
			count: 100,
			offset: 0
		},
		filters: [
			{
				column: 'quantity_per_box',
				operator: '<=',
				value: { Int: 12 }
			}
		],
		sorts: [
			{
				column: 'stock',
				order: 'ASC'
			},
			{
				column: 'name',
				order: 'ASC'
			}
		]
	};

	let current_item_list: InventoryItem[] = [];

	function refresh_item_list() {
		api_call('inventory/list', 'POST', current_item_list_req).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					current_item_list = data;
				});
			} else {
				console.log('Error fetching item list');
				// This will clear the list, which will present a loading element of some kind
				current_item_list = [];
			}
		});
	}

	refresh_item_list();
</script>

<h1>Inventory</h1>

<PermissionGuard permissions={['INVENTORY_READ']}>
	<button on:click={refresh_item_list}>Refresh</button>

	<table>
		<thead>
			<tr>
				<!-- IDs are DB column names -->
				<th id="id">ID</th>
				<th id="name">Name</th>
				<th id="price">Price</th>
				<th id="stock">Stock</th>
				<th id="quantity_per_box">Qty/Box</th>
			</tr>
		</thead>
		<tbody>
			{#each current_item_list as item}
				<tr>
					<td>{item.id}</td>
					<td>{item.name}</td>
					<td>{item.price}</td>
					<td>{item.stock}</td>
					<td>{item.quantity_per_box}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</PermissionGuard>
