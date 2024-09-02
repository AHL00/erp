<script lang="ts">
	import { goto } from '$app/navigation';

	import type { ExpenseReportRequest } from '$bindings/ExpenseReportRequest';
	import type { ExpenseReportFilters } from '$bindings/ExpenseReportFilters';
	import type { ExpenseReport } from '$bindings/ExpenseReport';
	import type { Expense } from '$bindings/Expense';

	import Loader from '../../../components/Loader.svelte';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { User } from '$bindings/User';

	let data: ExpenseReportRequest = {
		start_date: '',
		end_date: '',
		filters: []
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
	let report: ExpenseReport | null = null;

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
		api_call('reports/create/expense', 'POST', data)
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

			<label for="desc_search"
				>Description search:
				<input
					type="checkbox"
					on:change={(e) => {
						// If checked, add a filter
						// @ts-ignore
						if (e.target.checked) {
							data.filters.push({ DescriptionSearch: '' });
							data.filters = data.filters;
						} else {
							// If unchecked, remove the filter
							data.filters = data.filters.filter((filter) => {
								// If filter key is not DescriptionSearch, keep it
								return Object.keys(filter)[0] !== 'DescriptionSearch';
							});
						}
					}}
				/>

				{#if data.filters.find((filter) => filter['DescriptionSearch'] !== undefined)}
					<input
						type="text"
						on:input={(e) => {
							// @ts-ignore
							data.filters.find((filter) => filter['DescriptionSearch'] !== undefined)[
								'DescriptionSearch'
							] = e.target.value;
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
			<h1 class="text-2xl font-bold">Report</h1>
			<hr />
			<div class="flex flex-col space-y-3 w-full h-full">
				<h1 class="text-xl font-bold">Expenses</h1>

				<div class="flex flex-col space-y-3 w-full h-fit">
					{#each report.expenses as expense}
						<div class="flex flex-row space-x-3 w-full h-fit">
							<div class="flex flex-col">
								<div class="font-bold">ID</div>
								<div>
									{expense.id}
								</div>
							</div>
							<div class="flex flex-col">
								<div class="font-bold">User</div>
								<div>
									{expense.created_by_user.username}
								</div>
							</div>
							<div class="flex flex-col">
								<div class="font-bold">Date</div>
								<div>
									{new Date(expense.date_time).toLocaleDateString()}
								</div>
							</div>
							<div class="flex flex-col">
								<div class="font-bold">Description</div>
								<div>
									{expense.description}
								</div>
							</div>
							<div class="flex flex-col">
								<div class="font-bold">Amount</div>
								<div>
									{expense.amount}
								</div>
							</div>
						</div>
					{/each}
					<hr />
					<div class="flex flex-row space-x-3 w-full h-fit">
						<div class="flex flex-col">
							<div class="font-bold">Total</div>
							<div>
								{report.total_expenses}
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
