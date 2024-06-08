<script lang="ts">
	import { type ListRequest } from '$bindings/ListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
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
			display_name: 'ID',
			current_sort: 'ASC',
			edit_type: { type: 'number', data: { integer: true, range: [0, null], step: 1 } },
            edit_readonly: true
		},
		{
			api_name: 'name',
			display_name: 'Name',
			current_sort: null,
			edit_type: {
				type: 'string',
				data: {
					length_range: [1, 255],
					regex: null
				}
			},
            edit_readonly: false
		},
		{
			api_name: 'price',
			display_name: 'Price',
			current_sort: null,
			edit_type: {
				type: 'number',
				data: {
					integer: false,
					range: [0, null],
					step: 1
				}
			},
            edit_readonly: false
		},
		{
			api_name: 'stock',
			display_name: 'Stock',
			current_sort: null,
			edit_type: {
				type: 'number',
				data: {
					integer: true,
					range: [0, null],
					step: 1
				}
			},
            edit_readonly: false
		},
		{
			api_name: 'quantity_per_box',
			display_name: 'Qty/Box',
			current_sort: null,
			edit_type: {
				type: 'number',
				data: {
					integer: true,
					range: [0, null],
					step: 1
				}
			},
            edit_readonly: false
		}
	];

    let default_item: InventoryItem = {
        id: -1,
        name: 'New Item',
        price: "0.00",
        stock: 0,
        quantity_per_box: 1,
    };
</script>

<div class="h-full w-full flex flex-col justify-start overflow-hidden">
<!-- <h1 class="text-2xl text-center my-4">Inventory</h1> -->
<CrudPanel
	list_request={inventory_list_req}
	objects_list={inventory_list}
	crud_endpoint="inventory"
	read_perms={['INVENTORY_READ']}
	write_perms={['INVENTORY_WRITE']}
    create_default={default_item}
	{columns}
></CrudPanel>
</div>
