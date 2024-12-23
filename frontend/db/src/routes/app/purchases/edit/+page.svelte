<script lang="ts">
	import { api_call } from '$lib/backend';

	// Get query params
	import { onMount, onDestroy } from 'svelte';
	import Loader from '../../../../components/Loader.svelte';
	import type { PurchaseMeta } from '$bindings/PurchaseMeta';
	import type { PurchaseItem } from '$bindings/PurchaseItem';
	import type { PurchasePatchRequest } from '$bindings/PurchasePatchRequest';
	import type { PurchaseItemUpdateRequest } from '$bindings/PurchaseItemUpdateRequest';
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
	import { local_date_to_iso_utc, open_in_new_tab, utc_date_to_local_rounded } from '$lib';
	onMount(async () => {
		showNavbar.set(true);
	});

	let query_params = new URLSearchParams(window.location.search);
	let loader: FullscreenLoader;

	let purchase_id = query_params.get('id');

	let purchase_meta: PurchaseMeta;
	let purchase_meta_editing: PurchaseMeta;
	let currently_saving_meta: boolean = false;

	const purchase_date_time_accuracy = 'second';

	let supplier_search_dropdown: SearchDropdown<Customer>;
	let supplier_display_map_fn = (val: Customer) => {
		return val.name;
	};
	let supplier_search_results: Customer[] = [];

	let edit_meta_save = async () => {
		currently_saving_meta = true;

		let purchase_patch_req: PurchasePatchRequest = {
			amount_paid: null,
			notes: null,
			supplier_id: null,
			date_time: null
		};

		if (purchase_meta.notes !== purchase_meta_editing.notes) {
			purchase_patch_req.notes = purchase_meta_editing.notes;
		}

		if (purchase_meta.amount_paid !== purchase_meta_editing.amount_paid) {
			purchase_patch_req.amount_paid = purchase_meta_editing.amount_paid;
		}

		if (purchase_meta_editing.supplier === null) {
			purchase_patch_req.supplier_id = null;
		} else if (
			purchase_meta.supplier !== null &&
			purchase_meta.supplier.id !== purchase_meta_editing.supplier.id
		) {
			purchase_patch_req.supplier_id = purchase_meta_editing.supplier.id;
		}

		if (purchase_meta.date_time !== purchase_meta_editing.date_time) {
			purchase_patch_req.date_time = purchase_meta_editing.date_time;
		}

		// This means wholesale, a supplier is chosen.
		if (purchase_meta_editing.supplier) {
			// If both are non-null and don't match, update
			if (purchase_meta.supplier) {
				if (purchase_meta.supplier.id !== purchase_meta_editing.supplier.id) {
					purchase_patch_req.supplier_id = purchase_meta_editing.supplier.id;
				}
			}

			// If the original supplier was null, update
			if (!purchase_meta.supplier) {
				purchase_patch_req.supplier_id = purchase_meta_editing.supplier.id;
			}
		}

		api_call(`purchases/${purchase_id}`, 'PATCH', purchase_patch_req)
			.then((res) => {
				if (!res) {
					toast.push('Failed to update purchase info');
					console.error('No response from server');
					currently_saving_meta = false;
					return;
				}

				if (res?.ok) {
					currently_saving_meta = false;

					let prepull_editing_meta = { ...purchase_meta_editing };

					// Pull latest purchase info just to be sure that it was actually updated
					// If the status was ok, it should be updated, but just to be sure
					load_info();

					if (compare_purchase_meta(prepull_editing_meta, purchase_meta)) {
						toast.push('Failed to update purchase info');
						console.error(
							'Failed to update purchase info, patch was successful but purchase info was not updated'
						);
						return;
					}

					toast.push('Purchase info saved successfully');
				} else {
					toast.push('Failed to update purchase info');
					console.error('Failed to update purchase');
				}
			})
			.catch((err) => {
				toast.push('Failed to update purchase info');
				console.error(err);
				currently_saving_meta = false;
			});

		currently_saving_meta = false;
	};

	let edit_items_save = async () => {
		currently_saving_items = true;

		let update_requests: PurchaseItemUpdateRequest[] = [];

		for (let i = 0; i < purchase_items_editing.length; i++) {
			let item = purchase_items_editing[i].purchase_item;

			let purchase_item_id;

			if (item.id < 0) {
				purchase_item_id = null;
			} else {
				purchase_item_id = item.id;
			}

			let item_update_req: PurchaseItemUpdateRequest = {
				purchase_item_id,
				inventory_item_id: item.inventory_item.id,
				price: item.price,
				quantity: item.quantity
			};

			update_requests.push(item_update_req);
		}

		// Make sure only one purchase item exists per inventory item
		for (let i = 0; i < update_requests.length; i++) {
			for (let j = i + 1; j < update_requests.length; j++) {
				if (update_requests[i].inventory_item_id === update_requests[j].inventory_item_id) {
					toast.push('Duplicate inventory items in purchase');
					currently_saving_items = false;
					return;
				}
			}
		}

		api_call(`purchases/${purchase_id}/items/update/preview`, 'POST', update_requests)
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

					api_call(`purchases/${purchase_id}/items/update`, 'POST', update_requests)
						.then((res) => {
							if (!res) {
								toast.push('Failed to update purchase items');
								console.error('No response from server');
								currently_saving_items = false;
								return;
							}

							if (res?.ok) {
								currently_saving_items = false;

								let prepull_editing_items = [...purchase_items_editing];

								// Pull latest purchase items just to be sure that it was actually updated
								// If the status was ok, it should be updated, but just to be sure
								load_items();

								if (
									compare_purchase_items(
										prepull_editing_items.map((x) => x.purchase_item),
										purchase_items
									)
								) {
									toast.push('Failed to update purchase items');
									console.error(
										'Failed to update purchase items, patch was successful but purchase items were not updated'
									);
									return;
								}

								toast.push('Purchase items saved successfully');
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
								toast.push('Failed to update purchase items');
								console.error('Failed to update purchase items');
							}
						})
						.catch((err) => {
							toast.push('Failed to update purchase items');
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
				toast.push('Failed to update purchase items');
				console.error(err);
				currently_saving_items = false;
			});
	};

	let loading_info: boolean = false;
	let loading_info_error: string | null = null;
	let loading_info_retry: boolean = false;

	// pif stands for purchase info field
	let pif_supplier: SearchDropdown<Customer>;
	let pif_amount_paid: HTMLInputElement;
	let pif_notes: HTMLTextAreaElement;
	let pif_date_time: HTMLInputElement;

	function set_pifs(x: PurchaseMeta) {
		// Spread to avoid copying the object as a reference
		purchase_meta_editing = { ...x };

		if (pif_supplier) {
			if (x.supplier) {
				pif_supplier.set_selected_value(x.supplier);
			} else {
				pif_supplier.remove_selected_value();
			}
		}

		if (pif_amount_paid !== undefined) {
			pif_amount_paid.value = x.amount_paid.toString();
		}

		if (pif_notes !== undefined) {
			pif_notes.value = x.notes;
		}

		if (pif_date_time !== undefined) {
			pif_date_time.value = utc_date_to_local_rounded(x.date_time, purchase_date_time_accuracy);
		}
	}

	function load_info() {
		if (loading_info) {
			console.error('Already loading purchase info');
			return;
		}

		loading_info = true;
		api_call(`purchases/${purchase_id}`, 'GET', null)
			.then((res) => {
				if (res === undefined) {
					loading_info_error = 'Failed to fetch purchase info';
					loading_info_retry = true;
					loading_info = false;
					console.error('Failed to fetch purchase info, no response from server');
					return;
				}

				if (res.status !== 200) {
					loading_info_error = 'Failed to fetch purchase info';
					loading_info_retry = true;
					loading_info = false;
					console.error('Failed to fetch purchase info, http status: ' + res.status);
					return;
				}

				res
					.json()
					.then((data) => {
						loading_info = false;
						purchase_meta = data;

						set_pifs(purchase_meta);

						loading_info_error = null;
						loading_info_retry = false;
					})
					.catch((err) => {
						loading_info_error = 'Failed to parse purchase info';
						loading_info_retry = true;
						console.error(err);
						loading_info = false;
					});
			})
			.catch((err) => {
				loading_info_error = 'Failed to fetch purchase info';
				loading_info_retry = true;
				console.error(err);
				loading_info = false;
			});
	}

	let purchase_items: PurchaseItem[] = [];

	type PurchaseItemEditingData = {
		inventory_item_search_results: InventoryItem[];
		/// Note: inventory_item may be null and id will be -n if new item
		purchase_item: PurchaseItem;
	};

	let purchase_items_editing: PurchaseItemEditingData[] = [];
	let inventory_item_search_results: InventoryItem[][] = [];
	let currently_saving_items: boolean = false;

	let loading_items: boolean = false;
	let loading_items_error: string | null = null;
	let loading_items_retry: boolean = false;

	function set_purchase_items_editing(x: PurchaseItem[]) {
		// For the number of items, create a new array of empty arrays
		let y: PurchaseItemEditingData[] = [];

		x.map((item) => {
			y.push({
				inventory_item_search_results: [],
				purchase_item: { ...item }
			});
		});

		purchase_items_editing = y;
	}

	function load_items() {
		if (loading_items) {
			console.error('Already loading purchase items');
			return;
		}

		loading_items = true;
		api_call(`purchases/${purchase_id}/items`, 'GET', null)
			.then((res) => {
				if (res === undefined) {
					loading_items_error = 'Failed to fetch purchase items';
					console.error('Failed to fetch purchase items');
					loading_items = false;
					loading_items_retry = true;
					return;
				}

				if (res.status !== 200) {
					loading_items_error = 'Failed to fetch purchase items';
					console.error('Failed to fetch, http status: ' + res.status);
					loading_items = false;
					loading_items_retry = true;
					return;
				}

				res
					.json()
					.then((data) => {
						loading_items = false;
						purchase_items = data;

						set_purchase_items_editing(purchase_items);

						loading_items_error = null;
						loading_items_retry = false;
					})
					.catch((err) => {
						loading_items_error = 'Failed to parse purchase items';
						loading_items = false;
						console.error(err);
						loading_items_retry = true;
					});
			})
			.catch((err) => {
				console.error(err);
				loading_items = false;
				loading_items_error = 'Failed to fetch purchase items';
				loading_items_retry = true;
			});
	}

	async function create_new_purchase_item() {
		// Find next id, should be smallest current id - 1
		// IDs must be unique for svelte keyed each block
		let smallest_id = purchase_items_editing
			.map((x) => x.purchase_item.id)
			.reduce((a, b) => Math.min(a, b), 0);

		purchase_items_editing.push({
			inventory_item_search_results: [],
			purchase_item: {
				id: smallest_id - 1,
				// @ts-ignore
				inventory_item: null,
				price: '0.00',
				quantity: 1
			}
		});

		// Reactivity
		purchase_items_editing = purchase_items_editing;
	}

	onMount(() => {
		loader.hide();

		if (purchase_id === null || purchase_id === undefined || purchase_id === '') {
			loader.set_text('No purchase ID provided');
			loader.disable_ellipsis();
			loader.icon = 'error';
			loader.show();
			console.error('No purchase ID provided');
			return;
		}

		load_info();
		load_items();
	});

	function compare_purchase_meta(a: PurchaseMeta, b: PurchaseMeta) {
		if (a === undefined || b === undefined) {
			return false;
		}

		let supplier_same;

		if (!a.supplier && !b.supplier) {
			supplier_same = true;
		} else if (!a.supplier || !b.supplier) {
			supplier_same = false;
		} else {
			supplier_same = a.supplier.id === b.supplier.id;
		}

		// Compare every field except id
		return (
			a.amount_paid === b.amount_paid &&
			supplier_same &&
			a.notes === b.notes &&
			utc_date_to_local_rounded(a.date_time, purchase_date_time_accuracy) ===
				utc_date_to_local_rounded(b.date_time, purchase_date_time_accuracy)
		);
	}

	function compare_purchase_item(a: PurchaseItem, b: PurchaseItem) {
		if (a === undefined || b === undefined) {
			return false;
		}

		return (
			a.id === b.id &&
			a.inventory_item.id === b.inventory_item.id &&
			parseFloat(a.price) === parseFloat(b.price) &&
			a.quantity === b.quantity
		);
	}

	/// Check if two arrays of purchase items are the same.
	/// The purchase of the items does not matter.
	function compare_purchase_items(a: PurchaseItem[], b: PurchaseItem[]) {
		if (a.length !== b.length) {
			return false;
		}

		let found_indexes: number[] = [];

		for (let i = 0; i < a.length; i++) {
			for (let j = 0; j < b.length; j++) {
				if (!found_indexes.includes(j) && compare_purchase_item(a[i], b[j])) {
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
		// If error, retry fetching purchase
		if (loading_info_retry) {
			setTimeout(() => {
				console.log('Retrying to load purchase info');
				loading_info_retry = false;
				load_info();
			}, 2500);
		}

		if (loading_items_retry) {
			setTimeout(() => {
				console.log('Retrying to load purchase items');
				loading_items_retry = false;
				load_items();
			}, 2500);
		}
	}
</script>

<svelte:head>
	<title>Purchase {purchase_id}</title>
</svelte:head>

<FullscreenLoader bind:this={loader} />
<form
	class="relative w-full h-full flex"
	on:submit|preventDefault
	id="purchase-edit-form"
	on:submit={() => {
		if (!compare_purchase_meta(purchase_meta, purchase_meta_editing)) {
			edit_meta_save();
		}

		if (
			!compare_purchase_items(
				purchase_items,
				purchase_items_editing.map((x) => x.purchase_item)
			)
		) {
			edit_items_save();
		}
	}}
>
	<PermissionGuard permissions={['PURCHASE_READ', 'PURCHASE_UPDATE']}>
		<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
			<div
				class="h-fit w-full p-3 space-y-3 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<div class="flex flex-row w-full justify-between items-center">
					<span class="text-2xl">Purchase Information</span>
				</div>

				<div class="flex flex-row h-fit w-full items-start space-x-3 relative">
					{#if loading_info}
						<div class="w-full h-full my-3 absolute left-0 -top-4 z-30">
							<Loader
								blur_background={true}
								icon={'dots'}
								icon_size={1.1}
								ellipsis={true}
								text={'Loading purchase info'}
							/>
						</div>
					{:else if currently_saving_meta}
						<div class="w-full h-full my-3 absolute left-0 -top-4 z-30">
							<Loader
								blur_background={true}
								icon={'dots'}
								icon_size={1.1}
								ellipsis={true}
								text={'Saving purchase info'}
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
						{#if purchase_meta_editing}
							<SearchDropdown
								classes={''}
								input_id="supplier"
								input_placeholder="Customer"
								search_endpoint="suppliers/search"
								search_perms={['CUSTOMERS_READ']}
								search_results={supplier_search_results}
								display_map_fn={supplier_display_map_fn}
								search_column="name"
								form_id="purchase-edit-form"
								validity_message={'Select a supplier from the dropdown'}
								required={true}
								initial_value={purchase_meta_editing.supplier}
								on_change={(value) => {
									if (purchase_meta_editing !== undefined) {
										purchase_meta_editing.supplier = value;
									}
								}}
								bind:this={pif_supplier}
							>
								<!-- Show supplie   r info here, should be found in pif_supplier.selected_value -->
								<div class="flex flex-col space-y-2 p-2">
									<span class="text-lg">Customer info</span>
									<span class="text-md">Name: {pif_supplier.selected_value()?.name}</span>
									<span class="text-md">Phone: {pif_supplier.selected_value()?.phone}</span>
									<span class="text-md">Address: {pif_supplier.selected_value()?.address}</span>
									{#if pif_supplier.selected_value()?.notes !== null}
										<span class="text-md">Notes: {pif_supplier.selected_value()?.notes}</span>
									{/if}
								</div>
							</SearchDropdown>
						{/if}
						<div class="flex flex-row w-full space-x-3 h-fit">
							<textarea
								class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
								placeholder="Purchase notes"
								on:input={() => {
									if (purchase_meta_editing !== undefined) {
										purchase_meta_editing.notes = pif_notes.value;
									}
								}}
								bind:this={pif_notes}
							></textarea>
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
								if (purchase_meta_editing !== undefined) {
									purchase_meta_editing.amount_paid = pif_amount_paid.value;
								}
							}}
							bind:this={pif_amount_paid}
						/>

						<div class="flex flex-row w-full justify-end space-x-3">
							<input
								type="datetime-local"
								step="1"
								class="flex-grow box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
								placeholder="Date and time"
								on:input={() => {
									if (purchase_meta_editing !== undefined) {
										purchase_meta_editing.date_time = local_date_to_iso_utc(pif_date_time.value);
									}
								}}
								bind:this={pif_date_time}
							/>
							{#if !compare_purchase_meta(purchase_meta, purchase_meta_editing)}
								<button
									class="bg-red-500 text-white px-2 py-1 rounded-md"
									on:click={() => {
										set_pifs(purchase_meta);
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
							text={'Loading purchase info'}
						/>
					</div>
				{:else if currently_saving_items}
					<div class="w-full h-full absolute left-0 z-30">
						<Loader
							blur_background={true}
							icon={'dots'}
							icon_size={1.1}
							ellipsis={true}
							text={'Saving purchase info'}
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
					<table class="w-full border-separate border-spacing-x-3 border-spacing-y-1">
						<thead>
							<tr>
								<th class="p-2 z-20 w-max bg-custom-lighter dark:bg-custom-dark">Item</th>
								<th class="p-2 z-20 w-28 bg-custom-lighter dark:bg-custom-dark italic">Qty/Box</th>
								<th class="p-2 z-20 w-36 bg-custom-lighter dark:bg-custom-dark italic">Stock</th>
								<th class="p-2 z-20 w-28 bg-custom-lighter dark:bg-custom-dark">Qty</th>
								<th class="p-2 z-20 w-36 bg-custom-lighter dark:bg-custom-dark">Price</th>
								<th class="p-2 z-20 w-36 bg-custom-lighter dark:bg-custom-dark">Total</th>
								<!-- <th class="z-20 bg-custom-lighter dark:bg-custom-dark"></th> -->
							</tr>
						</thead>
						<tbody>
							{#each purchase_items_editing as data, i (data.purchase_item.id)}
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
											initial_value={data.purchase_item.inventory_item}
											form_id="purchase-edit-form"
											validity_message={'Select an item from the dropdown'}
											required={true}
											on_change={(value) => {
												if (value === null) {
													return;
												}

												data.purchase_item.price = value.price;
												data.purchase_item.inventory_item = value;

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
											value={data.purchase_item.inventory_item
												? data.purchase_item.inventory_item.quantity_per_box
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
												value={data.purchase_item.inventory_item
													? data.purchase_item.inventory_item.stock
													: null}
											/>
											{#if !purchase_items[i]}
												<span
													class="w-min box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												>
													{(data.purchase_item.quantity < 0 ? '' : '+') + data.purchase_item.quantity}
												</span>
											{:else if data.purchase_item.inventory_item && data.purchase_item.quantity !== purchase_items[i].quantity}
												<span
													class="w-min box-border border border-dashed italic dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												>
													{(-purchase_items[i].quantity + data.purchase_item.quantity < 0 ? '' : '+') +
														(-purchase_items[i].quantity + data.purchase_item.quantity)}
												</span>
											{/if}
										</div>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Quantity"
											form="purchase-edit-form"
											min="1"
											bind:value={data.purchase_item.quantity}
										/>
									</td>
									<td>
										<div class="space-x-2 flex">
											<input
												type="number"
												class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
												placeholder="Price"
												form="purchase-edit-form"
												min="0"
												bind:value={data.purchase_item.price}
											/>
											{#if data.purchase_item.inventory_item && parseFloat(data.purchase_item.inventory_item.price) != parseFloat(data.purchase_item.price)}
												<button
													class="w-min box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
													on:click={() => {
														// truncate the price string to 2 decimal places
														let price = parseFloat(data.purchase_item.inventory_item.price);

														data.purchase_item.price = price.toFixed(2);
													}}
												>
													<i class="fas fa-sync"></i>
												</button>
											{/if}
										</div>
									</td>
									<td>
										<input
											type="number"
											class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
											placeholder="Total"
											form="purchase-edit-form"
											value={Math.round(
												parseFloat(data.purchase_item.price) * data.purchase_item.quantity * 100
											) / 100}
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
													purchase_items_editing = purchase_items_editing.filter(
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
						on:click={create_new_purchase_item}
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
							text={'Saving purchase items'}
						/>
					</div>
				{/if}

				<button
					form="purchase-edit-form"
					type="submit"
					id="save-purchase-button"
					bind:this={save_button}
					class="bg-green-500
                    text-white px-2 py-1 rounded-md
                    disabled:opacity-40 disabled:cursor-not-allowed
                    "
					disabled={!(
						!compare_purchase_meta(purchase_meta, purchase_meta_editing) ||
						!compare_purchase_items(
							purchase_items,
							purchase_items_editing.map((x) => x.purchase_item)
						) ||
						currently_saving_meta ||
						currently_saving_items
					)}
				>
					<i class="fas fa-save pr-1"></i>
					Save
				</button>

				<div class="flex flex-col flex-grow justify-center items-end space-y-1">
					<span
						class="text-md text-custom-text-light-darker dark:text-custom-text-dark-lighter font-bold"
						>Total</span
					>

					<CurrencySpan
						custom_class="text-2xl"
						value={purchase_items_editing.reduce(
							(acc, x) => acc + parseFloat(x.purchase_item.price) * x.purchase_item.quantity,
							0
						)}
					/>
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
</style>
