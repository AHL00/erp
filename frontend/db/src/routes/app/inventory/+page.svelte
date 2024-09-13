<script lang="ts">
	import { type ListRequest } from '$bindings/ListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import { type InventoryItemPostRequest } from '$bindings/InventoryItemPostRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../../components/crud/types';

	let inventory_list_req: ListRequest = {
		range: {
			count: 100,
			offset: 0
		},
		filters: [],
		sorts: [
			{
				column: 'id',
				order: 'ASC'
			}
		]
	};

	let inventory_list: InventoryItem[] = [];

	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			api_request_name: null,
			display_name: 'ID',
			display_map_fn: null,
			current_sort: 'ASC',
			type: { type: 'number', data: { integer: true, range: [0, null], step: 1 } },
			edit: true,
			readonly: true
		},
		{
			api_name: 'name',
			api_request_name: null,
			display_name: 'Name',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'string',
				data: {
					length_range: [1, 255],
					regex: null
				}
			},
			edit: true,
			readonly: false
		},
		{
			api_name: 'price',
			api_request_name: null,
			display_name: 'Price',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'currency'
			},
			edit: true,
			readonly: false
		},
		{
			api_name: 'stock',
			api_request_name: null,
			display_name: 'Stock',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'number',
				data: {
					integer: true,
					range: [null, null],
					step: 1
				}
			},
			edit: true,

			readonly: false
		},
		{
			api_name: 'quantity_per_box',
			api_request_name: null,
			display_name: 'Qty/Box',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'number',
				data: {
					integer: true,
					range: [0, null],
					step: 1
				}
			},
			edit: true,
			readonly: false
		}
	];

	let default_item: InventoryItemPostRequest = {
		name: 'New Item',
		price: '0.00',
		stock: 0,
		quantity_per_box: 1
	};
</script>

<div class="flex flex-col h-full">
	<CrudPanel
		list_request={inventory_list_req}
		objects_list={inventory_list}
		crud_endpoint="inventory"
		read_perms={['INVENTORY_READ']}
		write_perms={['INVENTORY_WRITE']}
		create_post_request={default_item}
		{columns}
	></CrudPanel>
</div>
