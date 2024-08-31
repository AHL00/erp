<script lang="ts">
	import { redirect } from '$lib';
	import type { ListRequest } from '$bindings/ListRequest';
	import type { OrderMeta } from '$bindings/OrderMeta';
	import type { CrudColumn } from '../../../components/crud/types';
	import type { Customer } from '$bindings/Customer';
	import type { User } from '$bindings/User';
	import type { OrderPostRequest } from '$bindings/OrderPostRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import SearchDropdown from '../../../components/SearchDropdown.svelte';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';

	let order_list_req: ListRequest = {
		range: {
			count: 25,
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

	let orders_list: OrderMeta[] = [];

	// Won't be editing so no need for edit config in columns
	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			api_request_name: null,
			display_name: 'ID',
			display_map_fn: null,
			current_sort: null,
			edit_type: { type: 'none' },
			edit_readonly: true
		},
		{
			api_name: 'date_time',
			api_request_name: null,
			display_name: 'Date',
			display_map_fn: (val: string) => {
				let date = new Date(val);
				return date.toLocaleString();
			},
			current_sort: 'DESC',
			edit_type: { type: 'none' },
			edit_readonly: true
		},
		{
			api_name: 'customer',
			api_request_name: 'customers.name',
			display_name: 'Customer',
			display_map_fn: (val: Customer) => {
				return val.name;
			},
			current_sort: null,
			edit_type: { type: 'none' },
			edit_readonly: true
		},
		{
			api_name: 'created_by_user',
			api_request_name: 'users.username',
			display_name: 'Created by',
			display_map_fn: (val: User) => {
				return val.username;
			},
			current_sort: null,
			edit_type: { type: 'none' },
			edit_readonly: true
		},
		{
			api_name: 'amount_paid',
			api_request_name: null,
			display_name: 'Amount paid',
			display_map_fn: null,
			current_sort: null,
			edit_type: { type: 'none' },
			edit_readonly: true
		},
		{
			api_name: 'retail',
			api_request_name: null,
			display_name: 'Order type',
			display_map_fn: (val: boolean) => {
				return val ? 'Retail' : 'Wholesale';
			},
			current_sort: null,
			edit_type: { type: 'none' },
			edit_readonly: true
		}
	];

	let customer_search_results: Customer[] = [];
	let customer_display_map_fn = (val: Customer) => {
		return val.name;
	};

	let customer_search_dropdown: SearchDropdown<Customer>;

	let currently_creating: boolean = false;

	let create_submit_callback = async (e: any) => {
		e.preventDefault();
		currently_creating = true;

		let customer = customer_search_dropdown.selected_value();

		if (!customer_search_dropdown.reportValidity()) {
			currently_creating = false;
			return;
		}

		let customer_id = customer?.id;

		if (!customer_id) {
			currently_creating = false;
			return;
		}

		let order_type = document.querySelector('input[name="order_type"]:checked');

		// @ts-ignore
		let order_type_val = order_type.value;

		// @ts-ignore
		let notes = document.querySelector('textarea').value;

		let order_create_req: OrderPostRequest = {
			amount_paid: '0.0',
			customer_id: customer_id,
			notes: notes,
			retail: order_type_val === 'retail'
		};

		api_call('orders', 'POST', order_create_req)
			.then((res) => {
				if (!res) {
					toast.push('Failed to create order');
					console.error('No response from server');
					currently_creating = false;
					return;
				}

				if (res?.ok) {
					res
						.json()
						.then((data) => {
							redirect(`/app/orders/edit?id=${data}`);
							currently_creating = false;
						})
						.catch((err) => {
							toast.push('Failed to parse response after creating order');
							console.error(err);
						});
				} else {
					toast.push('Failed to create order');
					console.error(res, res.status);
					currently_creating = false;
				}
			})
			.catch((err) => {
				console.error(err);
				toast.push('Failed to create order');

				currently_creating = false;
			});
	};
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<PermissionGuard permissions={['ORDER_WRITE']}>
		<div class="w-full flex flex-row h-fit space-x-3">
			<div
				class="h-fit w-1/2 p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<div class="flex flex-row w-full justify-between items-center p-2">
					<span class="text-2xl">New order</span>
					{#if !currently_creating}
						<button
							class="bg-green-500 text-white px-2 py-1 rounded-md place-self-end justify-self-end self-end"
							type="submit"
							form="order-create-form"
							disabled={currently_creating}
						>
							Create order
						</button>
					{/if}
				</div>

				<form
					class="flex flex-col h-fit w-full p-2 items-start space-y-3"
					id="order-create-form"
					on:submit={create_submit_callback}
				>
					<SearchDropdown
						input_id="customer"
						input_placeholder="Customer"
						search_endpoint="customers/search"
						search_perms={['CUSTOMERS_READ']}
						search_results={customer_search_results}
						display_map_fn={customer_display_map_fn}
						search_column="name"
						search_count={10}
						form_id="order-create-form"
						validity_message={'Select a customer from the dropdown'}
						required={true}
						bind:this={customer_search_dropdown}
					/>
					<div class="flex flex-row w-full space-x-3 h-fit">
						<textarea
							class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
							placeholder="Notes"
						></textarea>

						<div
							class="w-fit flex flex-col space-y-1 py-2 px-4 border dark:border-custom-dark-outline border-custom-light-outline rounded"
						>
							<span class="text-md">Order type:</span>
							<label class="flex flex-row items-center space-x-2">
								<input type="radio" name="order_type" value="retail" />
								<span class="text-md font-thin">Retail</span>
							</label>
							<label class="flex flex-row items-center space-x-2">
								<input type="radio" name="order_type" value="wholesale" checked />
								<span class="text-md font-thin">Wholesale</span>
							</label>
						</div>
					</div>
				</form>
			</div>
			<div
				class="w-1/2 p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			></div>
		</div>
	</PermissionGuard>
	<PermissionGuard permissions={['ORDER_READ']}>
		<div
			class="w-full rounded-lg p-1 h-full shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
		>
			<span class="text-2xl m-3">Past orders</span>
			<CrudPanel
				list_request={order_list_req}
				objects_list={orders_list}
				crud_endpoint="orders"
				read_perms={['ORDER_READ']}
				write_perms={['ORDER_WRITE']}
				create_default={null}
				edit_override={(item_id) => {
					redirect(`/app/orders/edit?id=${item_id}`);
				}}
				delete_enabled={true}
				{columns}
			></CrudPanel>
		</div>
	</PermissionGuard>
</div>
