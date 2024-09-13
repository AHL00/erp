<script lang="ts">
	import { type ListRequest } from '$bindings/ListRequest';
	import { type Customer } from '$bindings/Customer';
	import { type CustomerPostRequest } from '$bindings/CustomerPostRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../../components/crud/types';

	let customers_list_req: ListRequest = {
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

	let customers_list: Customer[] = [];

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

	let create_default: CustomerPostRequest = {
		name: 'New Customer',
		phone: '09 ',
		address: '',
		notes: ''
	};
</script>

<div class="flex flex-col h-full">
	<CrudPanel
		list_request={customers_list_req}
		objects_list={customers_list}
		crud_endpoint="customers"
		read_perms={['CUSTOMERS_READ']}
		write_perms={['CUSTOMERS_WRITE']}
		create_post_request={create_default}
		{columns}
	></CrudPanel>
</div>
