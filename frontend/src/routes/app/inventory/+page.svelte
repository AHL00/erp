<script lang="ts">
	import { type InventoryItemListRequest } from '$bindings/InventoryItemListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import CrudTable from '../../../components/crud/CrudTable.svelte';
	import type { CrudColumn } from '../../../components/crud/types';
	import type { CrudEditType } from '../../../components/crud/types';
	import Loader from '../../../components/Loader.svelte';

	let current_item_list_req: InventoryItemListRequest = {
		range: {
			count: 100,
			offset: 0
		},
		filters: [],
		sorts: []
	};

	let current_item_list: InventoryItem[] = [];

	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			display_name: 'ID',
			current_sort: null,
			edit_type: { type: 'hidden' }
		},
		{
			api_name: 'name',
			display_name: 'Name',
			current_sort: null,
			edit_type: {
				type: 'string',
				data: {
					length_range: [1, 255],
					text_area: false,
					regex: null
				}
			}
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
			}
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
			}
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
			}
		}
	];
</script>

<CrudTable
	page_title="Inventory"
	list_request={current_item_list_req}
	objects_list={current_item_list}
	crud_endpoint="inventory"
	read_perms={['INVENTORY_READ']}
	write_perms={['INVENTORY_WRITE']}
	{columns}
></CrudTable>
