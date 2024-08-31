<script lang="ts">
	import { goto } from '$app/navigation';

	import type { ReportRequest } from '$bindings/ReportRequest';
	import type { ReportFilters } from '$bindings/ReportFilters';
	import type { ReportRequestType } from '$bindings/ReportRequestType';
	import type { Report } from '$bindings/Report';

	import Loader from '../../../components/Loader.svelte';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';

	const rt_variants = ['Revenue', 'Profit', 'Expenses', 'Product', 'Receivable', 'Payable'];

	let data: ReportRequest = {
		start_date: '',
		end_date: '',
		filters: [],
		report_types: ['Revenue', 'Profit', 'Expenses']
	};
	// One mon
	let start_date = new Date().toDateString();
	let end_date = new Date().toDateString();

	$: {
		data.end_date = new Date(end_date).toISOString();
	}

	$: {
		data.start_date = new Date(start_date).toISOString();
	}

	let currently_generating_report = false;
	let report: Report | null = null;

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
		api_call('reports/create', 'POST', data)
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
			{#each rt_variants as rt_variant}
				<label
					><input
						type="checkbox"
						bind:group={data.report_types}
						value={rt_variant}
					/>{rt_variant}</label
				>
			{/each}
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
			<h1 class="text-2xl font-bold">Report</h1>
			<hr />
			<div class="flex flex-col space-y-3 w-full h-full">
				<h1 class="text-xl font-bold">Orders</h1>
				{#each report.orders as order}
					<div class="flex flex-row space-x-3 w-full h-fit">
						<div class="flex flex-col">
							<div class="font-bold">ID</div>
							<div>
								{order.id}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold">Customer</div>
							<div>
								{order.customer.name}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold">Date</div>
							<div>
								{new Date(order.date_time).toLocaleDateString()}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold">Total</div>
							<div>
								{order.items.reduce((acc, item) => acc + item.price * item.quantity, 0)}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold">Paid</div>
							<div>
								{order.amount_paid}
							</div>
						</div>
						<div class="flex flex-col">
							<div class="font-bold">Receivable</div>
							<div>
								{Math.max((order.items.reduce((acc, item) => acc + item.price * item.quantity, 0) -
									order.amount_paid), 0)}
							</div>
						</div>
					</div>
				{/each}
				<hr />
				<div class="flex flex-row space-x-3 w-full h-fit">
					{#each Object.keys(report.data) as key}
						<div class="flex flex-col">
							<div class="font-bold">{key}</div>
							<div>
								{parseFloat(report.data[key])}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{:else}
			<div class="flex-grow">Nothing generated yet...</div>
		{/if}
	</div>
</div>
