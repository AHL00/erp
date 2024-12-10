<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { api_call, get_setting } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { Order } from '$bindings/Order';
	import type { Customer } from '$bindings/Customer';
	import type { InventoryItem } from '$bindings/InventoryItem';
	import Loader from '$lib/../components/Loader.svelte';
	import PermissionGuard from '../../../../components/PermissionGuard.svelte';
	import FullscreenLoader from '../../../../components/FullscreenLoader.svelte';
	import { showNavbar } from '../../../../stores/navbarStore';
	import type { OrderItem } from '$bindings/OrderItem';
	import CurrencySpan from '../../../../components/currency/CurrencySpan.svelte';
	import { utc_iso_to_local_formatted } from '$lib';

	let order_id: string | null = null;
	let order: Order | null = null;
	let order_items: OrderItem[] = [];
	let loading = true;
	let filling_data_count = 0;

	let should_print = false;

	const query_params = new URLSearchParams(window.location.search);
	should_print = query_params.get('print') === 'true';

	onMount(async () => {
		showNavbar.set(false);

		order_id = query_params.get('id');
		if (order_id) {
			await fetchOrder(order_id);
		} else {
			toast.push('Order ID not found in URL');
		}
	});

	let logo_high_res_uri = '';
	filling_data_count++;
	get_setting('logo_high_resolution')
		.then((res) => {
			// @ts-ignore
			logo_high_res_uri = res['ImageBase64URI'];
			filling_data_count--;
		})
		.catch((error) => {
			toast.push('Failed to fetch logo');
			filling_data_count--;
		});

	let business_name: string | null = null;
	filling_data_count++;
	get_setting('business_name')
		.then((res) => {
			// @ts-ignore
			business_name = res['Text'];
			filling_data_count--;
		})
		.catch((error) => {
			toast.push('Failed to fetch business name');
			filling_data_count--;
		});

	let business_address: string | null = null;
	filling_data_count++;
	get_setting('business_address')
		.then((res) => {
			// @ts-ignore
			business_address = res['Text'];
			filling_data_count--;
		})
		.catch((error) => {
			toast.push('Failed to fetch business address');
			filling_data_count--;
		});

	let business_phone_nums: string[] | null = null;
	filling_data_count++;
	get_setting('business_phone_numbers')
		.then((res) => {
			// @ts-ignore
			business_phone_nums = res['TextVec'];
			filling_data_count--;
		})
		.catch((error) => {
			toast.push('Failed to fetch business phone numbers');
			filling_data_count--;
		});

	let business_bank_accounts: string[] | null = null;
	filling_data_count++;
	get_setting('business_bank_accounts')
		.then((res) => {
			// @ts-ignore
			business_bank_accounts = res['TextVec'];
			filling_data_count--;
		})
		.catch((error) => {
			toast.push('Failed to fetch business bank accounts');
			filling_data_count--;
		});

	$: {
		console.log(filling_data_count);
	}
	$: {
		if (should_print && order && filling_data_count === 0) {
			console.log('printing');
			setTimeout(() => {
				window.print();
				window.onafterprint = () => {
					console.log('after print');
					window.close();
				};
			}, 1000);
		}
	}

	async function fetchOrder(id: string) {
		try {
			const response = await api_call(`orders/${id}`, 'GET', null);

			if (!response) {
				toast.push('Failed to fetch order');
				return;
			}

			if (response.status === 200) {
				order = await response.json();
			} else {
				toast.push('Failed to fetch order');
			}

			const items_response = await api_call(`orders/${id}/items`, 'GET', null);

			if (!items_response) {
				toast.push('Failed to fetch order items');
				return;
			}

			if (items_response.status === 200) {
				order_items = await items_response.json();
			} else {
				toast.push('Failed to fetch order items');
			}
		} catch (error) {
			toast.push('Error fetching order');
			console.error(error);
		} finally {
			loading = false;
		}
	}

	let total = 0;
	$: {
		if (order && order_items) {
			console.log(order);
			total = order_items.reduce((acc, item) => acc + parseFloat(item.price) * item.quantity, 0);
		}
	}

	// Settings
	let invoice_signature_fields: boolean = false;

	get_setting('invoice_signature_fields').then((res) => {
		// @ts-ignore
		invoice_signature_fields = res.Boolean;
	});

	let date_time_fmt = 'dd/mm/yy hh:MM tt';
	get_setting('date_time_format').then((res) => {
		// @ts-ignore
		date_time_fmt = res.Text;
	});
</script>

<svelte:head>
	<PermissionGuard permissions={['ORDER_READ']}>
		{#if order?.retail}
			<title
				>Invoice #{order_id} Retail {utc_iso_to_local_formatted(order?.date_time, date_time_fmt)}
				{order?.retail_customer_name}</title
			>
		{:else if order}
			<title
				>Invoice #{order_id}
				{utc_iso_to_local_formatted(order?.date_time, date_time_fmt)}
				{order?.customer?.name}</title
			>
		{:else}
			<title>Invoice #{order_id}</title>
		{/if}
		<title slot="denied">Permission Denied</title>
	</PermissionGuard>
</svelte:head>

<PermissionGuard permissions={['ORDER_READ']}>
	{#if loading}
		<FullscreenLoader ellipsis={true} icon="dots" text="Loading Order" />
	{:else}
		<div class="w-full h-fit bg-white">
			<div class="flex flex-col justify-start w-full h-fit">
				<div class="flex flex-row justify-between w-full h-fit pb-5 break-inside-avoid">
					<div class="flex flex-col space-y-3">
						<object
							data={logo_high_res_uri}
							type="image/png"
							class="w-24 h-24 rounded-xl"
							aria-label="Logo"
						>
						</object>
						<div class="flex flex-row items-end space-x-3">
							<span class="text-3xl text-black font-sans font-light">Invoice</span>
							<span class="text-2xl text-zinc-800 font-sans font-light">#{order_id}</span>
						</div>
					</div>
					<div class="flex flex-col space-y-3 items-end justify-center">
						<span class="text-3xl text-black font-sans font-normal">{business_name}</span>
						<div class="flex flex-col items-end space-y-1">
							<span class="text-sm text-black font-sans font-light">{business_address}</span>
							<div class="flex flex-col items-end">
								{#if business_phone_nums}
									{@const phone_nums = business_phone_nums ?? []}
									{#each phone_nums as phone_num}
										<span class="text-sm text-black font-sans font-light">{phone_num}</span>
									{/each}
								{/if}
							</div>
						</div>
					</div>
				</div>
				<hr />
				<div class="flex flex-row justify-between w-full h-fit py-3 break-inside-avoid gap-x-4">
					<div class="flex-grow">
						{#if order?.retail}
							<div class="flex flex-col">
								<span class="text-sm text-zinc-700 font-sans font-bold">Customer</span>
								<span class="text-xl text-black font-sans font-light"
									>{order?.retail_customer_name}</span
								>
							</div>
						{:else}
							<div class="flex flex-col">
								<span class="text-md text-zinc-700 font-sans font-bold">Customer</span>
								<span class="text-xl text-black font-sans font-light">{order?.customer?.name}</span>
							</div>
						{/if}
						<div class="flex flex-wrap gap-x-3 gap-y-1 flex-grow place-items-center mt-2">
							{#if order?.retail}
								<!-- <span class="text-md text-black font-sans font-light">
                                TODO: Retail customer info</span
                                > -->
								{#if order?.retail_customer_address && order?.retail_customer_address.length > 0}
									<div class="flex flex-col">
										<span class="text-xs text-zinc-700 font-sans font-bold">Address</span>
										<span class="text-sm text-black font-sans font-light"
											>{order?.retail_customer_address}</span
										>
									</div>
								{/if}
								<div class="flex flex-col">
									<span class="text-xs text-zinc-700 font-sans font-bold">Phone</span>
									<span class="text-sm text-black font-sans font-light"
										>{order?.retail_customer_phone}</span
									>
								</div>
							{:else}
								<div class="flex flex-col">
									<span class="text-xs text-zinc-700 font-sans font-bold">Address</span>
									<span class="text-sm text-black font-sans font-light"
										>{order?.customer?.address}</span
									>
								</div>
								<div class="flex flex-col">
									<span class="text-xs text-zinc-700 font-sans font-bold">Phone</span>
									<span class="text-sm text-black font-sans font-light"
										>{order?.customer?.phone}</span
									>
								</div>
								{#if order?.customer?.notes.trim().length > 0}
									<div class="flex flex-col">
										<span class="text-xs text-zinc-700 font-sans font-bold">Notes</span>
										<span class="text-sm text-black font-sans font-light"
											>{order?.customer?.notes}</span
										>
									</div>
								{/if}
							{/if}
						</div>
					</div>
					<div class="flex flex-col items-end space-y-2">
						<div class="flex flex-col items-end">
							<span class="text-xs text-zinc-700 font-sans font-bold">Date</span>
							<span class="text-sm text-black font-sans font-light"
								>{utc_iso_to_local_formatted(
									order?.date_time ? order.date_time : new Date(0).toISOString(),
									date_time_fmt
								)}
							</span>
						</div>
						<div class="flex flex-col items-end">
							<span class="text-xs text-zinc-700 font-sans font-bold">Type</span>
							<span class="text-sm text-black font-sans font-light"
								>{order?.retail ? 'Retail' : 'Wholesale'}</span
							>
						</div>
					</div>
				</div>
				<hr />

				<div class="flex flex-row justify-center w-full h-fit py-4">
					<table class="table-auto w-full">
						<thead>
							<tr>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start">No.</th>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start">Item</th>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start">Qty</th>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start"
									>Qty/Box</th
								>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start">Price</th
								>
								<th class="px-2 py-1 text-sm text-zinc-700 font-sans font-bold text-start"
									>Amount</th
								>
							</tr>
						</thead>
						<tbody>
							{#each order_items as item, i}
								<tr class="my-2">
									<td class="px-2 py-1 text-sm text-black font-sans font-light">{i + 1}</td>
									<td
										class="px-2 flex flex-col {item.inventory_item.description.trim().length > 0
											? 'pt-1 pb-1'
											: 'py-1'}"
									>
										<span class="text-sm text-black font-sans font-light">
											{item.inventory_item.name}
										</span>
										{#if item.inventory_item.description}
											<span class="text-xs text-zinc-700 font-sans font-light">
												{item.inventory_item.description}
											</span>
										{/if}
									</td>
									<td class="px-2 py-1 text-sm text-black font-sans font-light">{item.quantity}</td>
									<td class="px-2 py-1 text-sm text-black font-sans font-light"
										>{item.inventory_item.quantity_per_box}</td
									>
									<td class="px-2 py-1 text-sm text-black font-sans font-light">
										<CurrencySpan value={parseFloat(item.price)} />
									</td>
									<td class="px-2 py-1 text-sm text-black font-sans font-light">
										<CurrencySpan value={parseFloat(item.price) * item.quantity} />
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<div class="h-fit w-full mt-auto break-inside-avoid">
					<hr />
					<div class="flex flex-row mt-4 space-x-4 justify-between">
						{#if order?.notes && order?.notes.length > 0}
							<div
								class="flex flex-col space-y-1 flex-initial {business_bank_accounts &&
								business_bank_accounts.length > 0
									? 'w-1/4'
									: 'w-1/2'}"
							>
								<span class="text-sm text-zinc-800 font-sans font-bold">Notes</span>
								<span class="text-xs text-black font-sans font-light whitespace-pre-line"
									>{order?.notes}</span
								>
							</div>
						{/if}
						{#if business_bank_accounts && business_bank_accounts.length > 0}
							<div
								class="flex flex-wrap flex-row place-content-start
                            {order?.notes && order?.notes.length > 0
									? 'flex-initial w-1/2'
									: 'flex-grow'}
                            "
							>
								<span class="w-full h-fit text-sm text-zinc-800 font-sans font-bold mb-1"
									>Bank Accounts</span
								>
								{#each business_bank_accounts as account}
									<span class="w-fit h-fit mr-2 text-xs text-black font-sans font-light"
										>{account}</span
									>
								{/each}
							</div>
						{/if}
						<div class="flex flex-col items-end space-y-1 flex-initial w-1/4">
							<span class="text-sm text-zinc-800 font-sans font-bold">Total</span>
							<CurrencySpan custom_class="text-2xl text-black font-sans font-light" value={total} />
						</div>
					</div>

					{#if invoice_signature_fields}
						<div class="flex flex-row justify-between mt-4">
							<div class="flex flex-col items-end">
								<div class="flex flex-col justify-center text-center">
									<span class="text-md text-[#bbbbbb] font-sans mt-8">______________________</span>
									<span class="text-sm text-zinc-700 font-sans mt-1">Checked by</span>
								</div>
							</div>
							<div class="flex flex-col items-end">
								<div class="flex flex-col justify-center text-center">
									<span class="text-md text-[#bbbbbb] font-sans mt-8">______________________</span>
									<span class="text-sm text-zinc-700 font-sans mt-1">Received by</span>
								</div>
							</div>
							<div class="flex flex-col items-end">
								<div class="flex flex-col justify-center text-center">
									<span class="text-md text-[#bbbbbb] font-sans mt-8">______________________</span>
									<span class="text-sm text-zinc-700 font-sans mt-1">Authorised by</span>
								</div>
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}
	<div slot="denied" class="flex justify-center w-screen h-screen">
		<FullscreenLoader
			ellipsis={false}
			icon="error"
			text="You do not have permission to view invoices"
		/>
	</div>
</PermissionGuard>

<style>
	* {
		border-color: #bbbbbb;
	}

	@page {
		size: A4;
	}
</style>
