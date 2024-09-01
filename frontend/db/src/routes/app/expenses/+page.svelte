<script lang="ts">
	import { type Expense } from '$bindings/Expense';
    import { type ExpensePostRequest } from '$bindings/ExpensePostRequest';
	import type { ListRequest } from '$bindings/ListRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../../components/crud/types';

	let expenses_list_req: ListRequest = {
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

	let expenses_list: Expense[] = [];

	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			api_request_name: null,
			display_name: 'ID',
			display_map_fn: null,
			current_sort: 'ASC',
			edit_type: { type: 'number', data: { integer: true, range: [0, null], step: 1 } },
			edit_readonly: true
		},
		{
			api_name: 'date_time',
			api_request_name: null,
			display_name: 'Date',
			display_map_fn: (date_time: string) => new Date(date_time).toLocaleString(),
			current_sort: null,
			edit_type: {
				type: 'datetime'
			},
			edit_readonly: true
		},
		{
			api_name: 'description',
			api_request_name: null,
			display_name: 'Description',
			display_map_fn: null,
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
			api_name: 'amount',
			api_request_name: null,
			display_name: 'Amount',
			display_map_fn: null,
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
	];

	let default_item: ExpensePostRequest = {
		amount: '0',
		description: ''
	};
</script>

<div class="flex flex-col h-full">
	<CrudPanel
		list_request={expenses_list_req}
		objects_list={expenses_list}
		crud_endpoint="expenses"
		read_perms={['EXPENSES_READ']}
		write_perms={['EXPENSES_WRITE']}
		create_post_request={default_item}
		{columns}
	></CrudPanel>
</div>
