<script lang="ts">
	import { api_call } from '$lib/backend';

	// Get query params
	import { onMount, onDestroy } from 'svelte';
	import Loader from '../../../../components/Loader.svelte';
	import type { OrderMeta } from '$bindings/OrderMeta';
	import type { OrderItem } from '$bindings/OrderItem';
	import type { OrderPatchRequest } from '$bindings/OrderPatchRequest';
	import { toast } from '@zerodevx/svelte-toast';
	import SearchDropdown from '../../../../components/SearchDropdown.svelte';
	import FullscreenLoader from '../../../../components/FullscreenLoader.svelte';
	import PermissionGuard from '../../../../components/PermissionGuard.svelte';
	import type { Customer } from '$bindings/Customer';

	let query_params = new URLSearchParams(window.location.search);
	let loader: FullscreenLoader;

	let order_id = query_params.get('id');

	let order_meta: OrderMeta;
	let order_meta_editing: OrderMeta;
	let currently_saving_meta: boolean = false;

	let customer_search_dropdown: SearchDropdown<Customer>;
	let customer_display_map_fn = (val: Customer) => {
		return val.name;
	};
	let customer_search_results: Customer[] = [];

	let edit_submit_callback = async (e: any) => {
		// Disable default form submission
		e.preventDefault();

		currently_saving_meta = true;

		let order_patch_req: OrderPatchRequest = {
			retail: null,
			amount_paid: null,
			notes: null,
			customer_id: null
		};

		if (order_meta.notes !== order_meta_editing.notes) {
			order_patch_req.notes = order_meta_editing.notes;
		}

		if (order_meta.amount_paid !== order_meta_editing.amount_paid) {
			order_patch_req.amount_paid = order_meta_editing.amount_paid;
		}

		if (order_meta.customer.id !== order_meta_editing.customer.id) {
			order_patch_req.customer_id = order_meta_editing.customer.id;
		}

		if (order_meta.retail !== order_meta_editing.retail) {
			order_patch_req.retail = order_meta_editing.retail;
		}

		api_call(`orders/${order_id}`, 'PATCH', order_patch_req)
			.then((res) => {
				if (!res) {
					toast.push('Failed to update order info');
					console.error('No response from server');
					currently_saving_meta = false;
					return;
				}

				if (res?.ok) {
					currently_saving_meta = false;

					let prepull_editing_meta = { ...order_meta_editing };

					// Pull latest order info just to be sure that it was actually updated
					// If the status was ok, it should be updated, but just to be sure
					load_info();

					if (compare_order_meta(prepull_editing_meta, order_meta)) {
						toast.push('Failed to update order info');
						console.error(
							'Failed to update order info, patch was successful but order info was not updated'
						);
						return;
					}

					toast.push('Order info saved successfully');
				} else {
					toast.push('Failed to update order info');
					console.error('Failed to update order');
				}
			})
			.catch((err) => {
				toast.push('Failed to update order info');
				console.error(err);
				currently_saving_meta = false;
			});
	};

	let loading_info: boolean = false;
	let loading_info_error: string | null = null;
	let loading_info_retry: boolean = false;

	// oif stands for order info field
	let oif_customer: SearchDropdown<Customer>;
	let oif_order_type_retail: HTMLInputElement;
	let oif_order_type_wholesale: HTMLInputElement;
	let oif_amount_paid: HTMLInputElement;
	let oif_notes: HTMLTextAreaElement;

	function set_oifs(x: OrderMeta) {
		// Spread to avoid copying the object as a reference
		order_meta_editing = { ...x };

		if (oif_customer !== undefined) {
			oif_customer.set_selected_value(x.customer);
		}

		if (oif_order_type_retail !== undefined && oif_order_type_wholesale !== undefined) {
			if (x.retail) {
				oif_order_type_retail.checked = true;
			} else {
				oif_order_type_wholesale.checked = true;
			}
		}

		if (oif_amount_paid !== undefined) {
			oif_amount_paid.value = x.amount_paid.toString();
		}

		if (oif_notes !== undefined) {
			oif_notes.value = x.notes;
		}
	}

	function set_order_items(x: OrderItem[]) {
		order_items_editing = { ...x };
	}

	function load_info() {
		if (loading_info) {
			console.error('Already loading order info');
			return;
		}

		loading_info = true;
		api_call(`orders/${order_id}`, 'GET', null)
			.then((res) => {
				if (res === undefined) {
					loading_info_error = 'Failed to fetch order info';
					loading_info_retry = true;
					loading_info = false;
					console.error('Failed to fetch order info, no response from server');
					return;
				}

				if (res.status !== 200) {
					loading_info_error = 'Failed to fetch order info';
					loading_info_retry = true;
					loading_info = false;
					console.error('Failed to fetch order info, http status: ' + res.status);
					return;
				}

				res
					.json()
					.then((data) => {
						loading_info = false;
						order_meta = data;

						set_oifs(order_meta);

						loading_info_error = null;
						loading_info_retry = false;
					})
					.catch((err) => {
						loading_info_error = 'Failed to parse order info';
						loading_info_retry = true;
						console.error(err);
						loading_info = false;
					});
			})
			.catch((err) => {
				loading_info_error = 'Failed to fetch order info';
				loading_info_retry = true;
				console.error(err);
				loading_info = false;
			});
	}

	let order_items: OrderItem[] = [];
	let order_items_editing: OrderItem[] = [];
	let currently_saving_items: boolean = false;

	let loading_items: boolean = false;
	let loading_items_error: string | null = null;
	let loading_items_retry: boolean = false;

	function load_items() {
		if (loading_items) {
			console.error('Already loading order items');
			return;
		}

		loading_items = true;
		api_call(`orders/${order_id}/items`, 'GET', null)
			.then((res) => {
				if (res === undefined) {
					loading_items_error = 'Failed to fetch order items';
					console.error('Failed to fetch order items');
					loading_items = false;
					loading_items_retry = true;
					return;
				}

				if (res.status !== 200) {
					loading_items_error = 'Failed to fetch order items';
					console.error('Failed to fetch, http status: ' + res.status);
					loading_items = false;
					loading_items_retry = true;
					return;
				}

				res
					.json()
					.then((data) => {
						loading_items = false;
						order_items = data;

						set_order_items(order_items);

						loading_items_error = null;
						loading_items_retry = false;
					})
					.catch((err) => {
						loading_items_error = 'Failed to parse order items';
						loading_items = false;
						console.error(err);
						loading_items_retry = true;
					});
			})
			.catch((err) => {
				console.error(err);
				loading_items = false;
				loading_items_error = 'Failed to fetch order items';
				loading_items_retry = true;
			});
	}

	async function create_new_order_item() {}

	onMount(() => {
		loader.hide();

		if (order_id === null || order_id === undefined || order_id === '') {
			loader.set_text('No order ID provided');
			loader.disable_ellipsis();
			loader.icon = 'error';
			loader.show();
			console.error('No order ID provided');
			return;
		}

		load_info();
		load_items();
	});

	function compare_order_meta(a: OrderMeta, b: OrderMeta) {
		if (a === undefined || b === undefined) {
			return false;
		}

		// Compare every field except id
		return (
			a.amount_paid === b.amount_paid &&
			a.customer.id === b.customer.id &&
			a.notes === b.notes &&
			a.retail === b.retail
		);
	}

	$: {
		// If error, retry fetching order
		if (loading_info_retry) {
			setTimeout(() => {
				console.log('Retrying to load order info');
				loading_info_retry = false;
				load_info();
			}, 2500);
		}

		if (loading_items_retry) {
			setTimeout(() => {
				console.log('Retrying to load order items');
				loading_items_retry = false;
				load_items();
			}, 2500);
		}
	}
</script>

<FullscreenLoader bind:this={loader} />
<div class="relative w-full h-full flex">
	<PermissionGuard permissions={['ORDERS_READ', 'ORDERS_WRITE']}>
		<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
			<div
				class="h-fit w-full p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<div class="flex flex-row w-full justify-between items-center p-2">
					<span class="text-2xl">Order Information</span>
				</div>

				<form
					class="flex flex-row h-fit w-full p-2 items-start space-x-3 relative"
					id="order-edit-form"
					on:submit={edit_submit_callback}
				>
					{#if loading_info}
						<div class="w-full h-full my-3 absolute left-0 -top-4 z-30">
							<Loader
								blur_background={true}
								icon={'dots'}
								icon_size={1.1}
								ellipsis={true}
								text={'Loading order info'}
							/>
						</div>
					{:else if currently_saving_meta}
						<div class="w-full h-full my-3 absolute left-0 -top-4 z-30">
							<Loader
								blur_background={true}
								icon={'dots'}
								icon_size={1.1}
								ellipsis={true}
								text={'Saving order info'}
							/>
						</div>
					{:else if loading_info_error !== null}
						<div class="w-full h-full my-3 absolute left-0 -top-4 z-30">
							<Loader
								blur_background={true}
								icon={'error'}
								icon_size={0.9}
								ellipsis={true}
								text={loading_info_error + ', retrying soon'}
							/>
						</div>
					{/if}

					<div class="flex flex-col w-1/2 h-fit space-y-3">
						<SearchDropdown
							input_id="customer"
							input_placeholder="Customer"
							search_endpoint="customers/search"
							search_perms={['CUSTOMERS_READ']}
							search_results={customer_search_results}
							display_map_fn={customer_display_map_fn}
							search_column="name"
							search_count={10}
							form_id="order-edit-form"
							validity_message={'Select a customer from the dropdown'}
							required={true}
							on_change={(value) => {
								if (order_meta_editing !== undefined) {
									order_meta_editing.customer = value;
								}
							}}
							bind:this={oif_customer}
						>
							<!-- <div slot="view">
                                <!-- Show customer info here, should be found in oif_customer.selected_value -->
							<div class="flex flex-col space-y-2 p-2">
								<span class="text-lg">Customer info</span>
								<span class="text-md">Name: {oif_customer.selected_value()?.name}</span>
								<span class="text-md">Phone: {oif_customer.selected_value()?.phone}</span>
								<span class="text-md">Address: {oif_customer.selected_value()?.address}</span>
								{#if oif_customer.selected_value()?.notes !== null}
									<span class="text-md">Notes: {oif_customer.selected_value()?.notes}</span>
								{/if}
							</div>
							-->
						</SearchDropdown>
						<div class="flex flex-row w-full space-x-3 h-fit">
							<textarea
								class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
								placeholder="Order notes"
								on:input={() => {
									if (order_meta_editing !== undefined) {
										order_meta_editing.notes = oif_notes.value;
									}
								}}
								bind:this={oif_notes}
							></textarea>

							<div
								class="w-fit flex flex-col space-y-1 py-2 px-4 border dark:border-custom-dark-outline border-custom-light-outline rounded"
							>
								<span class="text-md">Order type:</span>
								<label class="flex flex-row items-center space-x-2">
									<input
										type="radio"
										name="order_type"
										value="retail"
										on:input={() => {
											if (order_meta_editing !== undefined) {
												order_meta_editing.retail = true;
											}
										}}
										bind:this={oif_order_type_retail}
									/>
									<span class="text-md font-thin">Retail</span>
								</label>
								<label class="flex flex-row items-center space-x-2">
									<input
										type="radio"
										name="order_type"
										value="wholesale"
										on:input={() => {
											if (order_meta_editing !== undefined) {
												order_meta_editing.retail = false;
											}
										}}
										bind:this={oif_order_type_wholesale}
									/>
									<span class="text-md font-thin">Wholesale</span>
								</label>
							</div>
						</div>
					</div>
					<div class="flex flex-col w-1/2 h-fit space-y-3">
						<input
							type="number"
							class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
							placeholder="Amount paid"
							name="amount_paid"
							id="amount_paid"
							min="0"
							on:input={() => {
								if (order_meta_editing !== undefined) {
									order_meta_editing.amount_paid = oif_amount_paid.value;
								}
							}}
							bind:this={oif_amount_paid}
						/>

						<div class="flex flex-row w-full justify-end space-x-3">
							{#if !compare_order_meta(order_meta, order_meta_editing)}
								<button
									class="bg-green-500 text-white px-2 py-1 rounded-md"
									type="submit"
									form="order-edit-form"
								>
									<i class="fas fa-save"></i>
									Save
								</button>
								<button
									class="bg-red-500 text-white px-2 py-1 rounded-md"
									on:click={() => {
										set_oifs(order_meta);
									}}
								>
									<i class="fas fa-undo-alt"></i>
									Reset
								</button>
							{/if}
						</div>
					</div>
				</form>
			</div>

			<!-- Center section -->
			<div
				class="w-full relative rounded-lg p-1 h-full shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				{#if loading_items}
					<div class="w-full h-full absolute left-0 z-30">
						<Loader
							blur_background={true}
							icon={'dots'}
							icon_size={1.1}
							ellipsis={true}
							text={'Loading order info'}
						/>
					</div>
				{:else if currently_saving_items}
					<div class="w-full h-full absolute left-0 z-30">
						<Loader
							blur_background={true}
							icon={'dots'}
							icon_size={1.1}
							ellipsis={true}
							text={'Saving order info'}
						/>
					</div>
				{:else if loading_items_error !== null}
					<div class="w-full h-full absolute left-0 z-30">
						<Loader
							blur_background={true}
							icon={'error'}
							icon_size={0.9}
							ellipsis={true}
							text={loading_items_error  + ', retrying soon'}
						/>
					</div>
				{/if}

				<span class="text-2xl m-3">Items</span>

				<div class="flex flex-col grow">
					<div
						class="h-full
                        overflow-y-visible
                    "
					>
						{#each order_items as item, i}
							<span>{item}</span>
						{/each}
					</div>

					<div class="flex p-2">
						<button
							class="bg-green-500 text-white px-2 py-1 rounded-md"
							on:click={create_new_order_item}
						>
							Add new item
						</button>
					</div>
				</div>
			</div>

			<!-- Bottom section -->
			<div
				class="w-full h-fit p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark mb-3 flex flex-col"
			>
				<div class="h-40"></div>
			</div>
		</div>
	</PermissionGuard>
</div>
