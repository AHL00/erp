<script lang="ts">
	import { api_call } from '$lib/backend';

	// Get query params
	import { onMount, onDestroy } from 'svelte';
	import Loader from '../../../../components/Loader.svelte';
	import type { OrderMeta } from '$bindings/OrderMeta';
	import type { OrderItem } from '$bindings/OrderItem';
	import type { OrderPatchRequest } from '$bindings/OrderPatchRequest';
	import type { OrderItemUpdateRequest } from '$bindings/OrderItemUpdateRequest';
	import type { StockUpdate } from '$bindings/StockUpdate';
	import type { StockUpdateFactory } from '$bindings/StockUpdateFactory';
	import { toast } from '@zerodevx/svelte-toast';
	import SearchDropdown from '../../../../components/SearchDropdown.svelte';
	import FullscreenLoader from '../../../../components/FullscreenLoader.svelte';
	import PermissionGuard from '../../../../components/PermissionGuard.svelte';
	import type { Customer } from '$bindings/Customer';
	import type { InventoryItem } from '$bindings/InventoryItem';

	import { showNavbar } from '../../../../stores/navbarStore';
	import CurrencySpan from '../../../../components/currency/CurrencySpan.svelte';
	import { beforeNavigate } from '$app/navigation';
	import {
		local_date_to_iso_utc,
		open_in_new_tab,
		order_item_final,
		order_item_total,
		order_total,
		utc_date_to_local_rounded
	} from '$lib';
	onMount(async () => {
		showNavbar.set(true);
	});

	let query_params = new URLSearchParams(window.location.search);
	let loader: FullscreenLoader;

	let order_id = query_params.get('id');

	let order_meta: OrderMeta;
	let order_meta_editing: OrderMeta;
	let currently_saving_meta: boolean = false;

	const order_date_time_accuracy = 'second';

	let customer_search_dropdown: SearchDropdown<Customer>;
	let customer_display_map_fn = (val: Customer) => {
		return val.name;
	};
	let customer_search_results: Customer[] = [];

	let edit_meta_save = async () => {
		currently_saving_meta = true;

		let order_patch_req: OrderPatchRequest = {
			retail: null,
			amount_paid: null,
			notes: null,
			date_time: null,

			customer_id: null,
			set_customer_id_null: false,

			retail_customer_address: null,
			retail_customer_name: null,
			retail_customer_phone: null,
			set_retail_customer_null: false,

			fulfilled: null
		};

		if (order_meta.notes !== order_meta_editing.notes) {
			order_patch_req.notes = order_meta_editing.notes;
		}

		if (order_meta.amount_paid !== order_meta_editing.amount_paid) {
			order_patch_req.amount_paid = order_meta_editing.amount_paid;
		}

		if (order_meta_editing.customer === null) {
			order_patch_req.customer_id = null;
		} else if (
			order_meta.customer !== null &&
			order_meta.customer.id !== order_meta_editing.customer.id
		) {
			order_patch_req.customer_id = order_meta_editing.customer.id;
		}

		// This means wholesale, a customer is chosen.
		if (order_meta_editing.customer) {
			// If both are non-null and don't match, update
			if (order_meta.customer) {
				if (order_meta.customer.id !== order_meta_editing.customer.id) {
					order_patch_req.customer_id = order_meta_editing.customer.id;
				}
			}

			// If the original customer was null, update
			if (!order_meta.customer) {
				order_patch_req.customer_id = order_meta_editing.customer.id;
			}
		}

		order_patch_req.retail = order_meta_editing.retail;

		// This means retail, remove customer no matter what
		if (order_patch_req.retail) {
			order_patch_req.set_customer_id_null = true;

			// Set retail customer info
			order_patch_req.retail_customer_address = order_meta_editing.retail_customer_address;
			order_patch_req.retail_customer_name = order_meta_editing.retail_customer_name;
			order_patch_req.retail_customer_phone = order_meta_editing.retail_customer_phone;
		} else {
			// If wholesale, remove retail customer info
			order_patch_req.set_retail_customer_null = true;
		}

		if (order_meta.fulfilled !== order_meta_editing.fulfilled) {
			order_patch_req.fulfilled = order_meta_editing.fulfilled;
		}

		if (order_meta.date_time !== order_meta_editing.date_time) {
			order_patch_req.date_time = order_meta_editing.date_time;
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

		currently_saving_meta = false;
	};

	let edit_items_save = async () => {
		currently_saving_items = true;

		let update_requests: OrderItemUpdateRequest[] = [];

		for (let i = 0; i < order_items_editing.length; i++) {
			let item = order_items_editing[i].order_item;

			let order_item_id;

			if (item.id < 0) {
				order_item_id = null;
			} else {
				order_item_id = item.id;
			}

			let item_update_req: OrderItemUpdateRequest = {
				order_item_id,
				inventory_item_id: item.inventory_item.id,
				price: item.price,
				quantity: item.quantity,
				discount: item.discount,
				discount_percentage: item.discount_percentage
			};

			update_requests.push(item_update_req);
		}

		// Make sure only one order item exists per inventory item
		for (let i = 0; i < update_requests.length; i++) {
			for (let j = i + 1; j < update_requests.length; j++) {
				if (update_requests[i].inventory_item_id === update_requests[j].inventory_item_id) {
					toast.push('Duplicate inventory items in order');
					currently_saving_items = false;
					return;
				}
			}
		}

		api_call(`orders/${order_id}/items/update/preview`, 'POST', update_requests)
			.then(async (res) => {
				if (!res) {
					toast.push('Failed to get stock update preview');
					console.error('No response from server');
					currently_saving_items = false;
					return;
				}

				if (res?.ok) {
					let stock_updates: StockUpdateFactory[] = await res.json();

					let stock_updates_str = stock_updates
						.map((x) => {
							return `[${x.delta}]: ${x.inventory.name} (ID: ${x.inventory.id})`;
						})
						.join('\n');

					let confirmed = confirm(
						`Are you sure you want to save these changes?\n\n${stock_updates_str}`
					);

					if (!confirmed) {
						currently_saving_items = false;
						return;
					}

					api_call(`orders/${order_id}/items/update`, 'POST', update_requests)
						.then((res) => {
							if (!res) {
								toast.push('Failed to update order items');
								console.error('No response from server');
								currently_saving_items = false;
								return;
							}

							if (res?.ok) {
								currently_saving_items = false;

								let prepull_editing_items = [...order_items_editing];

								// Pull latest order items just to be sure that it was actually updated
								// If the status was ok, it should be updated, but just to be sure
								load_items();
                                load_info();

								if (
									compare_order_items(
										prepull_editing_items.map((x) => x.order_item),
										order_items
									)
								) {
									toast.push('Failed to update order items');
									console.error(
										'Failed to update order items, patch was successful but order items were not updated'
									);
									return;
								}

								toast.push('Order items saved successfully');
								currently_saving_items = false;

								res
									.json()
									.then((data) => {
										let stock_updates: StockUpdate[] = data;

										console.log(stock_updates);
									})
									.catch((err) => {
										console.error(err);
									});
							} else {
								toast.push('Failed to update order items');
								console.error('Failed to update order items');
							}
						})
						.catch((err) => {
							toast.push('Failed to update order items');
							console.error(err);
							currently_saving_items = false;
						});
				} else {
					toast.push('Failed to get stock update preview');
					console.error('Failed to get stock update preview');
					currently_saving_items = false;
				}
			})
			.catch((err) => {
				toast.push('Failed to update order items');
				console.error(err);
				currently_saving_items = false;
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
	let oif_fulfilled: HTMLInputElement;
	let oif_date_time: HTMLInputElement;

	let oif_retail_customer_address: HTMLTextAreaElement;
	let oif_retail_customer_name: HTMLInputElement;
	let oif_retail_customer_phone: HTMLInputElement;

	function set_oifs(x: OrderMeta) {
		// Spread to avoid copying the object as a reference
		order_meta_editing = { ...x };

		if (oif_customer) {
			if (x.customer) {
				oif_customer.set_selected_value(x.customer);
			} else {
				oif_customer.remove_selected_value();
			}
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

		if (oif_fulfilled !== undefined) {
			oif_fulfilled.checked = x.fulfilled;
		}

		if (oif_date_time !== undefined) {
			oif_date_time.value = utc_date_to_local_rounded(x.date_time, order_date_time_accuracy);
		}
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

	type OrderItemEditingData = {
		inventory_item_search_results: InventoryItem[];
		/// Note: inventory_item may be null and id will be -n if new item
		order_item: OrderItem;
	};

	let order_items_editing: OrderItemEditingData[] = [];
	let inventory_item_search_results: InventoryItem[][] = [];
	let currently_saving_items: boolean = false;

	let loading_items: boolean = false;
	let loading_items_error: string | null = null;
	let loading_items_retry: boolean = false;

	function set_order_items_editing(x: OrderItem[]) {
		// For the number of items, create a new array of empty arrays
		let y: OrderItemEditingData[] = [];

		x.map((item) => {
			y.push({
				inventory_item_search_results: [],
				order_item: { ...item }
			});
		});

		order_items_editing = y;
	}

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

						set_order_items_editing(order_items);

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

	async function create_new_order_item() {
		// Find next id, should be smallest current id - 1
		// IDs must be unique for svelte keyed each block
		let smallest_id = order_items_editing
			.map((x) => x.order_item.id)
			.reduce((a, b) => Math.min(a, b), 0);

		order_items_editing.push({
			inventory_item_search_results: [],
			order_item: {
				id: smallest_id - 1,
				// @ts-ignore
				inventory_item: null,
				price: '0.00',
				quantity: 1,
                discount: '0.00',
                discount_percentage: true
			}
		});

		// Reactivity
		order_items_editing = order_items_editing;
	}

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

		let customer_same;

		if (!a.customer && !b.customer) {
			customer_same = true;
		} else if (!a.customer || !b.customer) {
			customer_same = false;
		} else {
			customer_same = a.customer.id === b.customer.id;
		}

		// Compare every field except id
		return (
			a.amount_paid === b.amount_paid &&
			customer_same &&
			a.notes === b.notes &&
			a.retail === b.retail &&
			a.retail_customer_address === b.retail_customer_address &&
			a.retail_customer_name === b.retail_customer_name &&
			a.retail_customer_phone === b.retail_customer_phone &&
			a.fulfilled === b.fulfilled &&
			utc_date_to_local_rounded(a.date_time, order_date_time_accuracy) ===
				utc_date_to_local_rounded(b.date_time, order_date_time_accuracy) &&
			a.total === b.total
		);
	}

	function compare_order_item(a: OrderItem, b: OrderItem) {
		if (a === undefined || b === undefined) {
			return false;
		}

		return (
			a.id === b.id &&
			a.inventory_item.id === b.inventory_item.id &&
			parseFloat(a.price) === parseFloat(b.price) &&
			a.quantity === b.quantity &&
			parseFloat(a.discount) === parseFloat(b.discount) &&
			a.discount_percentage === b.discount_percentage
		);
	}

	/// Check if two arrays of order items are the same.
	/// The order of the items does not matter.
	function compare_order_items(a: OrderItem[], b: OrderItem[]) {
		if (a.length !== b.length) {
			return false;
		}

		let found_indexes: number[] = [];

		for (let i = 0; i < a.length; i++) {
			for (let j = 0; j < b.length; j++) {
				if (!found_indexes.includes(j) && compare_order_item(a[i], b[j])) {
					found_indexes.push(j);
					break;
				}
			}
		}

		if (found_indexes.length !== a.length) {
			return false;
		}

		return true;
	}

	let save_button: HTMLButtonElement;

	beforeNavigate(({ cancel }) => {
		if (!save_button.disabled) {
			// Remind user they have unsaved changes
			let confirmed = confirm('You have unsaved changes, are you sure you want to leave?');
			if (confirmed) {
			} else {
				cancel();
			}
		}
	});

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

<svelte:head>
	<title>Order {order_meta?.id}</title>
</svelte:head>

<FullscreenLoader bind:this={loader} />
<form
	class="relative w-full h-full flex"
	on:submit|preventDefault
	id="order-edit-form"
	on:submit={() => {
		if (!compare_order_meta(order_meta, order_meta_editing)) {
			edit_meta_save();
		}

		if (
			!compare_order_items(
				order_items,
				order_items_editing.map((x) => x.order_item)
			)
		) {
			edit_items_save();
		}
	}}
>
	<PermissionGuard permissions={['ORDER_READ', 'ORDER_UPDATE']}>
		<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
			<div
				class="h-fit w-full p-3 space-y-3 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<div class="flex flex-row w-full justify-between items-center">
					<span class="text-2xl">Order Information</span>
				</div>

				<div class="flex flex-row h-fit w-full items-start space-x-3 relative">
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
						{#if order_meta_editing}
							{#if !order_meta_editing.retail}
								<SearchDropdown
									classes={order_meta_editing.retail ? 'hidden!' : ''}
									input_id="customer"
									input_placeholder="Customer"
									search_endpoint="customers/search"
									search_perms={['CUSTOMERS_READ']}
									search_results={customer_search_results}
									display_map_fn={customer_display_map_fn}
									search_column="name"
									form_id="order-edit-form"
									validity_message={'Select a customer from the dropdown'}
									required={true}
									initial_value={order_meta_editing.customer}
									on_change={(value) => {
										if (order_meta_editing !== undefined) {
											order_meta_editing.customer = value;
										}
									}}
									bind:this={oif_customer}
								>
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
								</SearchDropdown>
							{:else}
								<div class="flex flex-col w-full space-y-3 h-fit">
									<div class="flex flex-row w-full space-x-3 h-fit">
										<input
											class="w-7/12 box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent disabled:opacity-40"
											placeholder="Retail customer"
											required
											bind:this={oif_retail_customer_name}
											bind:value={order_meta_editing.retail_customer_name}
										/>
										<input
											class="w-5/12 box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent disabled:opacity-40"
											placeholder="Phone"
											required
											bind:this={oif_retail_customer_phone}
											bind:value={order_meta_editing.retail_customer_phone}
										/>
									</div>
									<textarea
										class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
										placeholder="Address"
										bind:this={oif_retail_customer_address}
										bind:value={order_meta_editing.retail_customer_address}
									></textarea>
								</div>
							{/if}
						{/if}
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
						<div class="flex flex-row w-full justify-start space-x-3">
							<div
								class="w-fit flex place-items-center flex-row space-x-2 py-1 px-2 border dark:border-custom-dark-outline border-custom-light-outline rounded"
							>
								<span class="text-md">Fulfilled:</span>
								<input
									type="checkbox"
									class="w-4 h-4"
									on:change={() => {
										order_meta_editing.fulfilled = !order_meta_editing.fulfilled;
									}}
									bind:this={oif_fulfilled}
								/>
							</div>
							<input
								type="datetime-local"
								step="1"
								class="flex-grow box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
								placeholder="Date and time"
								on:input={() => {
									if (order_meta_editing !== undefined) {
										order_meta_editing.date_time = local_date_to_iso_utc(oif_date_time.value);
									}
								}}
								bind:this={oif_date_time}
							/>
						</div>
						<div class="flex flex-row w-full justify-end space-x-3">
							{#if !compare_order_meta(order_meta, order_meta_editing)}
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
				</div>
			</div>

			<!-- Center section -->
			<div
				class="w-full relative rounded-lg p-3 shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col grow min-h-0"
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
							text={loading_items_error + ', retrying soon'}
						/>
					</div>
				{/if}
				<div class="flex flex-col grow min-h-0 overflow-auto">
					<table class="w-full border-collapse border-spacing-x-3 border-spacing-y-1">
						<thead>
							<tr>
								<th class="p-2 z-20 w-max bg-custom-lighter dark:bg-custom-dark">Item</th>
								<th class="p-2 z-20 w-16 bg-custom-lighter dark:bg-custom-dark italic">Qty/Box</th>
								<th class="p-2 z-20 w-32 bg-custom-lighter dark:bg-custom-dark italic">Stock</th>
								<th class="p-2 z-20 w-24 bg-custom-lighter dark:bg-custom-dark">Qty</th>
								<th class="p-2 z-20 w-36 bg-custom-lighter dark:bg-custom-dark">Price</th>
								<th class="p-2 z-20 w-36 bg-custom-lighter dark:bg-custom-dark">Discount / Item</th>
								<th class="p-2 z-20 w-28 bg-custom-lighter dark:bg-custom-dark">Final</th>
								<th class="p-2 z-20 w-32 bg-custom-lighter dark:bg-custom-dark">Total</th>
								<!-- <th class="z-20 bg-custom-lighter dark:bg-custom-dark"></th> -->
							</tr>
						</thead>
						<tbody>
							{#each order_items_editing as data, i (data.order_item.id)}
								<tr class="h-12">
									<td>
										<SearchDropdown
											input_id="inventory_item_{i}"
											input_placeholder="Inventory item"
											search_endpoint="inventory/search"
											search_perms={['INVENTORY_READ']}
											search_results={data.inventory_item_search_results}
											display_map_fn={(val) => {
												return val.name;
											}}
											display_extra_map_fn={(val) => {
												if (val.description.trim().length !== 0) {
													return val.description;
												} else {
													return null;
												}
											}}
											search_column="name"
											initial_value={data.order_item.inventory_item}
											form_id="order-edit-form"
											validity_message={'Select an item from the dropdown'}
											required={true}
											on_change={(value) => {
												if (value === null) {
													return;
												}

												data.order_item.price = value.price;
												data.order_item.inventory_item = value;

												console.log('on change');
											}}
											presearch_fn={(search) => {
												search = search.replace(/-/g, ' ');
												return search;
											}}
											on_initial_value={(value) => {}}
										/>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Qty/Box"
											disabled
											value={data.order_item.inventory_item
												? data.order_item.inventory_item.quantity_per_box
												: null}
										/>
									</td>
									<td>
										<div class="flex flex-row gap-x-2">
											<input
												type="number"
												class="w-full box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												placeholder="Stock"
												disabled
												value={data.order_item.inventory_item
													? data.order_item.inventory_item.stock
													: null}
											/>
											{#if !order_items[i]}
												<span
													class="w-min box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												>
													{(-data.order_item.quantity < 0 ? '' : '+') + -data.order_item.quantity}
												</span>
											{:else if data.order_item.inventory_item && data.order_item.quantity !== order_items[i].quantity}
												<span
													class="w-min box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												>
													{(order_items[i].quantity - data.order_item.quantity < 0 ? '' : '+') +
														(order_items[i].quantity - data.order_item.quantity)}
												</span>
											{/if}
										</div>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Quantity"
											form="order-edit-form"
											min="1"
											bind:value={data.order_item.quantity}
										/>
									</td>
									<td>
										<div class="gap-x-2 flex">
											<input
												type="number"
												class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												placeholder="Price"
												form="order-edit-form"
												min="0"
												bind:value={data.order_item.price}
											/>
											{#if data.order_item.inventory_item && parseFloat(data.order_item.inventory_item.price) != parseFloat(data.order_item.price)}
												<button
													type="reset"
													class="w-min box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
													on:click={() => {
														// truncate the price string to 2 decimal places
														let price = parseFloat(data.order_item.inventory_item.price);

														data.order_item.price = price.toFixed(2);
													}}
												>
													<i class="fas fa-sync"></i>
												</button>
											{/if}
										</div>
									</td>
									<td>
										<div class="gap-x-2 flex">
											<input
												type="number"
												class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												placeholder="Discount"
												form="order-edit-form"
												min="0"
												bind:value={data.order_item.discount}
											/>
											<button
												type="button"
												class="w-min box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												on:click={() => {
													data.order_item.discount_percentage =
														!data.order_item.discount_percentage;
												}}
											>
												{#if data.order_item.discount_percentage}
													<i class="fas fa-percent"></i>
												{:else}
													<i class="fa-regular fa-money-bill-1"></i>
												{/if}
											</button>
										</div>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border border-dashed dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Final"
											form="order-edit-form"
											value={order_item_final(data.order_item)}
											readonly
											disabled
										/>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border border-dashed dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Total"
											form="order-edit-form"
											value={order_item_total(data.order_item)}
											readonly
											disabled
										/>
									</td>
									<td class="w-8">
										<div class="h-full w-full flex flex-row justify-center items-center space-x-2">
											<button
												class="bg-red-500 text-white px-2 py-1 rounded-md"
												on:click={() => {
													// Remove the item from the array
													order_items_editing = order_items_editing.filter(
														(_, index) => index !== i
													);
												}}
											>
												<i class="fas fa-trash"></i>
											</button>
										</div>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<div class="flex pt-3">
					<button
						class="bg-green-500 text-white px-2 py-1 rounded-md"
						on:click={create_new_order_item}
					>
						Add new item
					</button>
				</div>
			</div>

			<!-- Bottom section -->
			<div
				class="w-full h-fit p-3 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark mb-3 flex flex-row justify-start items-end space-x-3"
			>
				{#if currently_saving_items}
					<div class="w-full h-full absolute left-0 z-30">
						<Loader
							blur_background={true}
							icon={'dots'}
							icon_size={1.1}
							ellipsis={true}
							text={'Saving order items'}
						/>
					</div>
				{/if}

				<button
					id="view-order-button"
					class="bg-green-500 text-white px-2 py-1 rounded-md"
					on:click={() => {
						open_in_new_tab(`/app/orders/invoice?id=${order_id}`);
					}}
				>
					<i class="fas fa-file-invoice pr-1"></i>
					View invoice
				</button>

				<button
					form="order-edit-form"
					type="submit"
					id="save-order-button"
					bind:this={save_button}
					class="bg-green-500
                    text-white px-2 py-1 rounded-md
                    disabled:opacity-40 disabled:cursor-not-allowed
                    "
					disabled={!(
						!compare_order_meta(order_meta, order_meta_editing) ||
						!compare_order_items(
							order_items,
							order_items_editing.map((x) => x.order_item)
						) ||
						currently_saving_meta ||
						currently_saving_items
					)}
				>
					<i class="fas fa-save pr-1"></i>
					Save
				</button>

				<div class="flex flex-row flex-grow justify-end items-end gap-x-5">
					{#if order_meta}
						<div class="flex flex-col justify-center items-end gap-y-1">
							<span
								class="text-md text-custom-text-light-darker dark:text-custom-text-dark-lighter font-bold"
								>Saved Total</span
							>

							<CurrencySpan
								custom_class="text-2xl"
								value={parseFloat(order_meta.total ?? '-1.00')}
							/>
						</div>
					{/if}
					<div class="flex flex-col justify-center items-end gap-y-1">
						<span
							class="text-md text-custom-text-light-darker dark:text-custom-text-dark-lighter font-bold"
							>Current Total</span
						>

						<CurrencySpan
							custom_class="text-2xl"
							value={order_total(order_items_editing.map((x) => x.order_item))}
						/>
					</div>
				</div>
			</div>
		</div>
	</PermissionGuard>
</form>

<style>
	th {
		position: sticky;
		top: 0;
	}

	td {
		padding: 0.6rem 0.4rem;
	}
</style>
