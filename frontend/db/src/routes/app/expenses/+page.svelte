<script lang="ts">
	import { type Expense } from '$bindings/Expense';
	import { type ExpensePostRequest } from '$bindings/ExpensePostRequest';
	import type { ListRequest } from '$bindings/ListRequest';
	import { get_setting } from '$lib/backend';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import type { CrudColumn } from '../../../components/crud/types';

	import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
	onMount(async () => {
		showNavbar.set(true);
	});

	let expenses_list_req: ListRequest = {
		range: {
			count: 100,
			offset: 0
		},
		filters: [],
		sorts: [
			{
				column: 'date_time',
				order: 'DESC'
			}
		]
	};

	let expenses_list: Expense[] = [];

    
	let date_time_fmt = 'dd/mm/yy hh:MM tt';
	get_setting('date_time_format').then((res) => {
		// @ts-ignore
		date_time_fmt = res.Text;
	});

	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			api_request_name: null,
			display_name: 'ID',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'number',
				data: { integer: true, range: [0, null], step: 1 }
			},
			edit: true,
			readonly: true,
            searchable: false
		},
		{
			api_name: 'date_time',
			api_request_name: null,
			display_name: 'Date',
			display_map_fn: (date_time: string) => new Date(date_time).toLocaleString(),
			current_sort: 'DESC',
			type: {
				type: 'datetime',
                accuracy: 'second',
                format: date_time_fmt
			},
			edit: true,
			readonly: true,
            searchable: false
		},
		{
			api_name: 'description',
			api_request_name: null,
			display_name: 'Description',
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
			readonly: false,
            searchable: false
		},
		{
			api_name: 'amount',
			api_request_name: null,
			display_name: 'Amount',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'currency'
			},
			edit: true,
			readonly: false,
            searchable: false,
            align: 'right'
		}
	];

	let default_item: ExpensePostRequest = {
		amount: '0',
		description: ''
	};
</script>

<svelte:head>
	<title>Expenses</title>
</svelte:head>

<div class="flex flex-col w-full h-screen min-h-0 items-center p-2 space-y-3 overflow-hidden">
	<div
		class="w-full rounded-lg p-1 flex-grow shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col min-h-0"
	>
		<CrudPanel
			list_request={expenses_list_req}
			objects_list={expenses_list}
			crud_endpoint="expenses"
			read_perms={['EXPENSES_READ']}
			update_perms={['EXPENSES_UPDATE']}
            delete_perms={['EXPENSES_DELETE']}
            create_perms={['EXPENSES_CREATE']}
			create_post_request={default_item}
			delete_enabled={true}
			{columns}
		></CrudPanel>
	</div>
</div>
