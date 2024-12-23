<script lang="ts">
	import { open_in_new_tab, redirect } from '$lib';
	import type { ListRequest } from '$bindings/ListRequest';
	import type { PurchaseMeta } from '$bindings/PurchaseMeta';
	import type { CrudColumn } from '../../../components/crud/types';
	import type { Supplier } from '$bindings/Supplier';
	import type { User } from '$bindings/User';
	import type { PurchasePostRequest } from '$bindings/PurchasePostRequest';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import SearchDropdown from '../../../components/SearchDropdown.svelte';
	import { api_call, get_setting } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';

	import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
	import type { StockUpdate } from '$bindings/StockUpdate';
	onMount(async () => {
		showNavbar.set(true);
	});

	let purchase_list_req: ListRequest = {
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

	let purchases_list: PurchaseMeta[] = [];

	let date_time_fmt = 'dd/mm/yy hh:MM tt';
	get_setting('date_time_format').then((res) => {
		// @ts-ignore
		date_time_fmt = res.Text;
	});

	// Won't be editing so no need for edit config in columns
	let columns: CrudColumn[] = [
		{
			api_name: 'id',
			api_request_name: null,
			display_name: 'ID',
			display_map_fn: null,
			current_sort: null,
			type: { type: 'number', data: { integer: true, range: [0, null], step: 1 } },
			edit: true,
			readonly: true,
			searchable: true,
            search_nested: 'purchases.id'
		},
		{
			api_name: 'date_time',
			api_request_name: null,
			display_name: 'Date',
			display_map_fn: null,
			current_sort: 'DESC',
			type: { type: 'datetime', accuracy: 'second', format: date_time_fmt },
			edit: true,
			readonly: true,
			searchable: false
		},
		{
			api_name: 'supplier',
			api_request_name: 'suppliers.name',
			display_name: 'Supplier',
			display_map_fn: (val: Supplier) => {
				return val.name;
			},
			current_sort: null,
			type: { type: 'use_display_map_fn_and_no_edit' },
			edit: false,
			readonly: true,
			searchable: true,
			search_nested: 'suppliers.name'
		},
		{
			api_name: 'created_by_user',
			api_request_name: 'users.username',
			display_name: 'Created by',
			display_map_fn: (val: User) => {
				return val.username;
			},
			current_sort: null,
			type: { type: 'use_display_map_fn_and_no_edit' },
			edit: false,
			readonly: true,
			searchable: true,
			search_nested: 'users.username'
		},
		{
			api_name: 'amount_paid',
			api_request_name: null,
			display_name: 'Amount paid',
			display_map_fn: null,
			current_sort: null,
			type: {
				type: 'currency'
			},
			edit: true,
			readonly: true,
			searchable: false
		}
	];

	let supplier_search_results: Supplier[] = [];
	let supplier_display_map_fn = (val: Supplier) => {
		return val.name;
	};

	let supplier_search_dropdown: SearchDropdown<Supplier>;

	let currently_creating: boolean = false;

	let create_submit_callback = async (e: any) => {
		e.preventDefault();
		currently_creating = true;

		let supplier;

		if (supplier_search_dropdown) {
			if (!supplier_search_dropdown.reportValidity()) {
				currently_creating = false;
				return;
			}

			supplier = supplier_search_dropdown.selected_value();
		} else {
			toast.push('Failed to create purchase');
			console.error('Supplier search dropdown not found');
			currently_creating = false;
			return;
		}

		// @ts-ignore
		let notes = document.querySelector('textarea').value;

		let supplier_id: number;
		if (supplier) {
			supplier_id = supplier.id;
		} else {
			toast.push('Supplier not selected');
			console.error('Supplier not selected');
			currently_creating = false;
			return;
		}

		let purchase_create_req: PurchasePostRequest = {
			amount_paid: '0.0',
			supplier_id: supplier_id,
			notes: notes
		};

		api_call('purchases', 'POST', purchase_create_req)
			.then((res) => {
				if (!res) {
					toast.push('Failed to create purchase');
					console.error('No response from server');
					currently_creating = false;
					return;
				}

				if (res?.ok) {
					res
						.json()
						.then((data) => {
							redirect(`/app/purchases/edit?id=${data}`);
							currently_creating = false;
						})
						.catch((err) => {
							toast.push('Failed to parse response after creating purchase');
							console.error(err);
						});
				} else {
					toast.push('Failed to create purchase');
					console.error(res, res.status);
					currently_creating = false;
				}
			})
			.catch((err) => {
				console.error(err);
				toast.push('Failed to create purchase');

				currently_creating = false;
			});
	};

	function post_delete_callback(res: Response) {
		res
			.json()
			.then((data) => {
				let stock_upates: StockUpdate[] = data;

				let string = 'Stock updates:\n';
				for (let update of stock_upates) {
					string += `${update.inventory_id} ${update.delta}\n`;
				}

				console.log(string);
			})
			.catch((err) => {
				console.error(err);
			});
	}
</script>

<svelte:head>
	<title>Purchases</title>
</svelte:head>

<div class="flex flex-col w-full h-screen min-h-0 items-center p-3 space-y-3 overflow-hidden">
	<PermissionGuard permissions={['PURCHASE_CREATE']}>
		<div class="w-full flex flex-row h-fit space-x-3">
			<div
				class="h-fit w-1/2 p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<div class="flex flex-row w-full justify-between items-center p-2">
					<span class="text-2xl">New purchase</span>
					{#if !currently_creating}
						<button
							class="bg-green-500 text-white px-2 py-1 rounded-md place-self-end justify-self-end self-end"
							type="submit"
							form="purchase-create-form"
							disabled={currently_creating}
						>
							Create purchase
						</button>
					{/if}
				</div>

				<form
					class="flex flex-col h-fit w-full p-2 items-start space-y-3"
					id="purchase-create-form"
					on:submit={create_submit_callback}
				>
					<SearchDropdown
						input_id="supplier"
						input_placeholder="Supplier"
						search_endpoint="suppliers/search"
						search_perms={['SUPPLIERS_READ']}
						search_results={supplier_search_results}
						display_map_fn={supplier_display_map_fn}
						search_column="name"
						form_id="purchase-create-form"
						validity_message={'Select a supplier from the dropdown'}
						required={true}
						bind:this={supplier_search_dropdown}
					/>
					<div class="flex flex-row w-full space-x-3 h-fit">
						<textarea
							class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
							placeholder="Notes"
						></textarea>
					</div>
				</form>
			</div>
			<div
				class="w-1/2 p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			></div>
		</div>
	</PermissionGuard>
	<PermissionGuard permissions={['PURCHASE_READ']}>
		<div
			class="w-full rounded-lg p-1 flex-grow shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col min-h-0"
		>
			<span class="text-2xl m-3">Past purchases</span>
			<CrudPanel
				list_request={purchase_list_req}
				objects_list={purchases_list}
				crud_endpoint="purchases"
				read_perms={['PURCHASE_READ']}
                create_perms={['PURCHASE_CREATE']}
                update_perms={['PURCHASE_UPDATE']}
                delete_perms={['PURCHASE_DELETE']}
				{post_delete_callback}
				create_post_request={null}
				edit_override={(item_id) => {
					redirect(`/app/purchases/edit?id=${item_id}`);
				}}
				delete_enabled={true}
				custom_buttons={[]}
				{columns}
			></CrudPanel>
		</div>
	</PermissionGuard>
</div>
