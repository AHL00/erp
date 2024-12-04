<script lang="ts">
	import { type ListRequest } from '$bindings/ListRequest';
	import { type Supplier } from '$bindings/Supplier';
	import { type SupplierPostRequest } from '$bindings/SupplierPostRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../../components/crud/types';

	import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
	onMount(async () => {
		showNavbar.set(true);
	});

	let suppliers_list_req: ListRequest = {
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

	let suppliers_list: Supplier[] = [];

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
			api_name: 'phone',
			api_request_name: null,
			display_name: 'Phone',
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
			api_name: 'address',
			api_request_name: null,
			display_name: 'Address',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'string',
				data: {
					length_range: [1, null],
					regex: null
				}
			},
			edit: true,
			readonly: false
		},
		{
			api_name: 'notes',
			api_request_name: null,
			display_name: 'Notes',
			display_map_fn: null,
			// (val: string) => {
			//     if (val === null || val.length === 0) {
			//         return '-';
			//     }

			//     return val.length > 10 ? val.slice(0, 10) + '...' : val;
			// },
			current_sort: null,
			type: {
				type: 'string',
				data: {
					length_range: [1, null],
					regex: null
				}
			},
			edit: true,
			readonly: false
		}
	];

	let create_default: SupplierPostRequest = {
		name: 'New Supplier',
		phone: '',
		address: '',
		notes: ''
	};
</script>

<div class="flex flex-col h-full">
	<CrudPanel
		list_request={suppliers_list_req}
		objects_list={suppliers_list}
		crud_endpoint="suppliers"
		read_perms={['SUPPLIERS_READ']}
		write_perms={['SUPPLIERS_WRITE']}
		create_post_request={create_default}
		{columns}
	></CrudPanel>
</div>
