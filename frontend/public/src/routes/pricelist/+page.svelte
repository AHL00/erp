<script lang="ts">
	import { type ListRequest } from '$bindings/ListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import CrudPanel from '../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../components/crud/types';

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
			current_sort: 'ASC'
		},
		{
			api_name: 'name',
			api_request_name: null,
			display_name: 'Name',
			display_map_fn: null,
			current_sort: null
		},
		{
			api_name: 'price',
			api_request_name: null,
			display_name: 'Price',
			display_map_fn: null,
			current_sort: null
		}
	];
</script>

<svelte:head>
    <title>Price List</title>
</svelte:head>

<div class="h-[100vh] overflow-y-hidden">
	<CrudPanel
		list_request={inventory_list_req}
		objects_list={inventory_list}
		crud_endpoint="public/inventory"
        custom_margins="mt-3 mx-3"
		{columns}
	></CrudPanel>
</div>
