<script lang="ts">
	import { goto } from '$app/navigation';

	import type { OrderReportRequest } from '$bindings/OrderReportRequest';
	import type { OrderReportFilter } from '$bindings/OrderReportFilter';
	import type { OrderReport } from '$bindings/OrderReport';
	import type { Order } from '$bindings/Order';

	import Loader from '../../../components/Loader.svelte';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { User } from '$bindings/User';
	import { CollapsibleCard } from 'svelte-collapsible';

	let data: OrderReportRequest = {
		start_date: '',
		end_date: '',
		filters: []
	};

	let start_date = new Date().toDateString();
	let end_date = new Date().toDateString();

	$: {
		data.end_date = new Date(end_date).toISOString();
	}

	$: {
		data.start_date = new Date(start_date).toISOString();
	}

	let currently_generating_report = false;
	let report: OrderReport | null = null;

	async function generateReport() {
		if (currently_generating_report) {
			toast.push('Already generating a report');
			return;
		}

		if (data.start_date === '' || data.end_date === '') {
			toast.push('Please select a start and end date');
			return;
		}

		currently_generating_report = true;
		api_call('reports/create/order', 'POST', data)
			.then((res) => {
				if (!res) {
					toast.push('Failed to generate report');
					console.error('No response from server');
					return;
				}

				if (res?.ok) {
					res
						.json()
						.then((data) => {
							report = data;
						})
						.catch((err) => {
							toast.push('Failed to parse response after generating report');
							console.error(err);
						});
				} else {
					toast.push('Failed to generate report');
					console.error(res, res.status);
				}
			})
			.catch((err) => {
				console.error(err);
			})
			.finally(() => {
				currently_generating_report = false;
			});
	}
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<div
		class="h-fit w-full p-3 space-y-3 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
	>
		<div class="flex flex-row space-x-3">
			<label for="user_id"
				>User:
				<input
					type="checkbox"
					on:change={(e) => {
						// If checked, add a filter
						// @ts-ignore
						if (e.target.checked) {
							data.filters.push({ UserId: 0 });
							data.filters = data.filters;
						} else {
							// If unchecked, remove the filter
							data.filters = data.filters.filter((filter) => {
								// If filter key is not UserId, keep it
								return Object.keys(filter)[0] !== 'UserId';
							});
						}
					}}
				/>

				{#if data.filters.find((filter) => filter['UserId'] !== undefined)}
					<input
						type="text"
						on:input={(e) => {
							// @ts-ignore
							data.filters.find((filter) => filter['UserId'] !== undefined)['UserId'] = parseInt(
								e.target.value
							);
						}}
					/>
				{/if}
			</label>

			<label for="customer">
				Customer:
				<input
					type="checkbox"
					on:change={(e) => {
						// If checked, add a filter
						// @ts-ignore
						if (e.target.checked) {
							data.filters.push({ CustomerId: 0 });
							data.filters = data.filters;
						} else {
							// If unchecked, remove the filter
							data.filters = data.filters.filter((filter) => {
								// If filter key is not CustomerId, keep it
								return Object.keys(filter)[0] !== 'CustomerId';
							});
						}
					}}
				/>

				{#if data.filters.find((filter) => filter['CustomerId'] !== undefined)}
					<input
						type="text"
						on:input={(e) => {
							// @ts-ignore
							data.filters.find((filter) => filter['CustomerId'] !== undefined)['CustomerId'] =
								parseInt(e.target.value);
						}}
					/>
				{/if}
			</label>
		</div>
		<div class="flex flex-row space-x-3">
			<label for="start_date">Start date: <input type="date" bind:value={start_date} /></label>
			<label for="end_date">End date: <input type="date" bind:value={end_date} /></label>
		</div>
		<button
			form="order-edit-form"
			type="submit"
			id="save-order-button"
			on:click={() => {
				generateReport();
			}}
			class="bg-green-500 text-white px-2 py-1 rounded-md"
		>
			<i class="fas fa-newspaper"></i>
			Generate report
		</button>
	</div>

	<div
		class="h-full w-full p-3 space-y-3 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
	>
		{#if currently_generating_report}
			<Loader text="Generating report" icon_size={1} />
		{:else if report}
			<h1 class="text-3xl font-bold">Orders Report</h1>
			<hr class="dark:border-custom-dark-outline border-custom-light-outline m-0" />

			<div class="flex flex-col space-y-3 w-full h-full">
				<div class="flex flex-col space-y-3 w-full h-fit">
					<CollapsibleCard>
						<div
							slot="header"
							class="flex flex-row space-x-3 w-full h-fit collapsible-div-main"
							on:click={() => {
								document.querySelectorAll(`.collapsible-div-main i`).forEach((i) => {
									i.classList.toggle('collapse-active');
								});
							}}
						>
							<span class="font-bold text-2xl"
								><i class="fa-solid fa-chevron-right mr-2 collapse-active text-xl"></i> Orders</span
							>
						</div>
						<div slot="body" class="flex flex-col space-y-2 w-full h-fit">
							{#each report.orders as order}
								<div class="ml-5 mt-2 flex flex-row space-x-3 w-full h-fit">
									<div class="flex flex-col">
										<div class="font-bold">ID</div>
										<div>
											{order.id}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Created by</div>
										<div>
											{order.created_by_user.username}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Customer</div>
										<div>
											{order.customer ? order.customer.name : 'N/A'}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Date</div>
										<div>
											{new Date(order.date_time).toLocaleDateString()}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Amount</div>
										<div>
											{order.items
												.reduce((acc, item) => acc + parseFloat(item.price) * item.quantity, 0)
												.toFixed(2)}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Paid</div>
										<div>
											{order.amount_paid}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Fulfilled</div>
										<div>
											{order.fulfilled ? 'Yes' : 'No'}
										</div>
									</div>
									<div class="flex flex-col">
										<div class="font-bold">Retail</div>
										<div>
											{order.retail ? 'Yes' : 'No'}
										</div>
									</div>
								</div>
								<CollapsibleCard open={false}>
									<div
										slot="header"
										class="ml-5 flex flex-row space-x-3 w-full h-fit collapsible-div-{order.id}"
										on:click={() => {
											document.querySelectorAll(`.collapsible-div-${order.id} i`).forEach((i) => {
												i.classList.toggle('collapse-active');
											});
										}}
									>
										<span class="font-bold">
											<i class="fa-solid fa-chevron-right mr-2"></i> Items
										</span>
									</div>
									<div slot="body" class="ml-10 flex flex-col space-y-3 w-full h-fit">
										<div class="flex flex-col space-y-2 w-full h-fit">
											{#each order.items as item}
												<div class="flex flex-row space-x-3 w-full h-fit">
													<div class="flex flex-col">
														<div class="font-bold">Name</div>
														<div>
															{item.inventory_item.name}
														</div>
													</div>
													<div class="flex flex-col">
														<div class="font-bold">Quantity</div>
														<div>
															{item.quantity}
														</div>
													</div>
													<div class="flex flex-col">
														<div class="font-bold">Price</div>
														<div>
															{parseFloat(item.price)}
														</div>
													</div>
												</div>
											{/each}
										</div>
									</div>
								</CollapsibleCard>
								{#if order.id !== report.orders[report.orders.length - 1].id}
									<hr class="dark:border-custom-dark-outline border-custom-light-outline" />
								{/if}
							{/each}
						</div>
					</CollapsibleCard>
					<div class="flex flex-row space-x-3 w-full h-fit">
						<div class="flex flex-col">
							<div class="font-bold text-xl">Total revenue</div>
							<div class="text-lg">
								{parseFloat(report.total_revenue)}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold text-xl">Total receivable</div>
							<div class="text-lg">
								{parseFloat(report.total_receivable)}
							</div>
						</div>
					</div>
				</div>
			</div>
		{:else}
			<div class="flex-grow">Nothing generated yet...</div>
		{/if}
	</div>
</div>

{#if false}
	NOTE: Fix the removal of unused css which gets activated by JS
	<i class="collapse-active"></i>
{/if}

<style>
	i.collapse-active {
		transform: rotate(90deg);
	}

	i {
		transition: transform 0.3s;
	}
</style>
