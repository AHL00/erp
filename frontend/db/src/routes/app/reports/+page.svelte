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
        console.log("Converting end: ", end_date);
		data.end_date = new Date(end_date).toISOString();
	}

    $: {
        console.log("Converting start: ", start_date);
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
							console.log(data);
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
		<h1 class="text-2xl font-bold">Finance Report</h1>
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
		<input type="date" bind:value={start_date} />
		<input type="date" bind:value={end_date} />
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
			<div class="flex flex-row space-x-3">
				{#each Object.keys(report) as key}
					<div class="flex flex-col">
						<h2 class="text-lg font-bold">{key}</h2>

						<!-- <div class="flex flex-col">
							{#each Object.keys(report[key]) as subkey}
								<div class="flex flex-row">
									<h3 class="text-md font-bold">{subkey}</h3>

									<p>{report[key][subkey]}</p>
								</div>
							{/each}
						</div> -->
					</div>
				{/each}
			</div>
		{:else}
			<div class="flex-grow">Nothing generated yet...</div>
		{/if}
	</div>
</div>
