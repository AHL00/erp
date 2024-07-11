<!-- A generic CRUD page that can edit any specified object on the API. -->
<script lang="ts" generics="EntryType extends { id: number }">
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';
	import type { CrudColumn } from './types';
	import type { ListSort } from '$bindings/ListSort';
	import type { ListFilter } from '$bindings/ListFilter';
	import SideBar from '../SideBar.svelte';
	import type { InventoryItem } from '$bindings/InventoryItem';
	import type { ListRequest } from '$bindings/ListRequest';
	import Loader from '$lib/../components/Loader.svelte';
	import { onMount } from 'svelte';

	// List request type, e.g. InventoryItemListRequest
	export let list_request: ListRequest;
	// Returned type from list, e.g. InventoryItem
	export let objects_list: EntryType[];
	export let crud_endpoint: string;

	export let custom_margins: string = 'm-0';

	/// The default item to create when the create button is pressed.
	/// The ID field will be ignored, and the rest of the fields will be used to create the item.
	/// If null, the create button will be disabled.
	let loading_count = 0;

	let error = false;

	function get_api_request_name(column: CrudColumn): string {
		return column.api_request_name ?? column.api_name;
	}

	function refresh_list() {
		loading_count++;

		//TODO: Implement catch for every api_call
		api_call(`${crud_endpoint}/list`, 'POST', list_request)
			.then((res) => {
				if (res?.status == 200) {
					res.json().then((data) => {
						objects_list = data;
					});
					loading_count--;

					if (error) {
						error = false;
					}
				} else {
					console.error('Error fetching list');

					toast.push('Error fetching data');

					// Clear the list
					objects_list = [];

					loading_count--;
					error = true;
				}
			})
			.catch((e) => {
				console.error('Error fetching list');

				toast.push('Error fetching data');

				// Clear the list
				objects_list = [];

				loading_count--;
				error = true;
			});
	}

	// Refresh the list when the page is loaded
	refresh_list();

	export let columns: CrudColumn[];

	function sort_toggle(column_index: number) {
		let column = columns[column_index];

		if (column.current_sort == null) {
			column.current_sort = 'ASC';
		} else if (column.current_sort == 'ASC') {
			column.current_sort = 'DESC';
		} else {
			column.current_sort = null;
		}

		// Update the list request
		let sort_list_index = list_request.sorts.findIndex(
			(sort: ListSort) => sort.column == get_api_request_name(column)
		);

		// Remove the sort if it exists
		if (sort_list_index != -1) {
			list_request.sorts.splice(sort_list_index, 1);
		}

		// Add the sort
		if (column.current_sort != null) {
			let sort: ListSort = {
				column: get_api_request_name(column),
				order: column.current_sort
			};

			list_request.sorts.push(sort);
		}

		// Hack svelte reactivity
		columns = columns;

		// Refresh the list
		refresh_list();
	}

	function find_sort_index(column_api_name: string): number {
		return list_request.sorts.findIndex((sort: ListSort) => sort.column == column_api_name);
	}

	function get_field_of_item(item: EntryType, field: string): any {
		return (item as any)[field];
	}

	/// NOTE: Also handles errors
	async function get_total_count(): Promise<number> {
		loading_count++;
		let res;

		try {
			res = await api_call(`${crud_endpoint}/count`, 'GET', null);
		} catch (e) {
			console.error('Error fetching count');

			toast.push('Error fetching data');

			// Clear the list, which will show a loading element
			objects_list = [];

			loading_count--;
			return Promise.reject('Error fetching count: ' + e);
		}

		if (res?.status == 200) {
			let data = await res.json();

			loading_count--;
			return Promise.resolve(data);
		} else {
			console.error('Error fetching count');

			toast.push('Error fetching data');

			// Clear the list, which will show a loading element
			objects_list = [];

			loading_count--;
			return Promise.reject('Error fetching count: HTTP code ' + res?.status);
		}
	}

	function get_item_from_api(id: number): Promise<EntryType> {
		return api_call(`${crud_endpoint}/${id}`, 'GET', null)
			.then((res) => {
				if (res?.status == 200) {
					return Promise.resolve(res.json());
				} else {
					return Promise.reject('Error fetching item, HTTP code ' + res?.status);
				}
			})
			.catch((e) => {
				return Promise.reject('Error fetching item: ' + e);
			});
	}

	let current_page = 1;
	let items_per_page = list_request.range.count;

	// Update this whenever item is added or removed
	let page_count: number = 0;

	get_page_count().then((n) => {
		page_count = n;
	});

	/// Reloads data even if the page is the same
	function change_page(page: number) {
		if (page < 1 || page > page_count) {
			return;
		}

		list_request.range.offset = (page - 1) * items_per_page;
		list_request.range.count = items_per_page;
		refresh_list();
		current_page = page;
	}

	async function get_page_count(): Promise<number> {
		// Send request for total count
		let count: number;

		// No need to catch, get_total_count will handle it
		try {
			count = await get_total_count();
		} catch (e: any) {
			// get_total_count will have handled showing the error to the user
			return e;
		}

		if (count !== undefined || count !== null) {
			// Calculate the page count
			let page_count = Math.ceil(count / items_per_page);

			return page_count;
		} else {
			return 0;
		}
	}

	let page_select_input: HTMLInputElement;
</script>

<div
	class="relative flex flex-col flex-grow self-center w-full h-full
            bg-custom-lighter dark:bg-custom-dark
            "
>
	{#if loading_count > 0}
		<div class="absolute h-full w-full flex">
			<Loader icon_size={1} blur_background={true} custom_classes="rounded-2xl" />
		</div>
	{/if}
	{#if error}
		<div class="absolute h-full w-full flex">
			<Loader
				icon_size={1}
				blur_background={true}
				custom_classes="rounded-2xl"
				icon="error"
				text="Error fetching data."
				ellipsis={false}
			/>
		</div>
	{/if}
	<div class="overflow-auto h-full flex flex-col {custom_margins}">
		<table class="w-full">
			<thead>
				<tr>
					<!-- NOTE: This background color will have to be changed if the body's background color is changed.  
                            Inheriting is a mess so I'm just going to hardcode it. -->
					{#each columns as column, index}
						<th
							class="p-2 z-20 bg-custom-lighter dark:bg-custom-dark"
							on:click={() => {
								sort_toggle(index);
							}}
						>
							<span style="white-space: nowrap;">
								{column.display_name}
								{#if column.current_sort == 'ASC'}
                                <span class="text-xl">▲</span>
								{/if}
								{#if column.current_sort == 'DESC'}
                                <span class="text-xl">▼</span>
								{/if}
								{#if column.current_sort != null && list_request.sorts.length > 1}
									<span class="text-xs">{find_sort_index(get_api_request_name(column)) + 1}</span>
								{/if}
							</span>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody id="{crud_endpoint}_table_body">
				{#each objects_list as item}
					<tr class="text-center">
						{#each columns as column}
							<td class="p-2">
								{#if column.display_map_fn !== null}
									{column.display_map_fn(get_field_of_item(item, column.api_name))}
								{:else}
									{get_field_of_item(item, column.api_name)}
								{/if}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<!-- Table controls -->
	<div class="inline-flex px-2 pt-3 pb-2 justify-start">
		<div class="justify-self-end ml-auto mx-1">
			<div
				class="inline-flex rounded-md outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline
                        text-custom-text-light-lighter dark:text-custom-text-dark-lighter"
				role="group"
			>
				<button
					type="button"
					class="rounded-l-lg px-2 dark:bg-custom-dark hover:brightness-90 inline-flex items-center
                            outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline z-10"
					on:click={() => {
						page_select_input.value = (parseInt(page_select_input.value) - 1).toString();
						page_select_input.dispatchEvent(new Event('change'));
					}}
				>
					<i class="fas fa-chevron-left text-sm"></i>
				</button>
				<div class="px-5 inline-flex align-middle">
					<input
						class=" dark:bg-custom-dark inline-flex items-center w-6"
						value={current_page}
						type="text"
						inputmode="numeric"
						bind:this={page_select_input}
						on:keypress={(e) => {
							// If non-numeric key is pressed, prevent it
							if (!/^\d+$/.test(e.key) && e.key !== 'Enter') {
								e.preventDefault();
							}
						}}
						on:change={(e) => {
							// @ts-ignore
							let new_page = parseInt(e.target.value);

							// Make sure the value is still in bounds
							if (new_page >= 1 && new_page <= page_count) {
								change_page(new_page);
							} else {
								// Move it to the nearest bound
								if (new_page < 1) {
									change_page(1);
									// @ts-ignore
									e.target.value = 1;
								} else {
									change_page(page_count);
									// @ts-ignore
									e.target.value = page_count;
								}
							}
						}}
					/>
					<span class="text-sm flex place-self-center"> of {page_count}</span>
				</div>
				<button
					type="button"
					class="rounded-r-lg px-2 dark:bg-custom-dark hover:brightness-90 inline-flex items-center
                            outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline z-10"
					on:click={() => {
						page_select_input.value = (parseInt(page_select_input.value) + 1).toString();
						page_select_input.dispatchEvent(new Event('change'));
					}}
				>
					<i class="fas fa-chevron-right text-sm"></i>
				</button>
			</div>
		</div>
	</div>
</div>

<style>
	th {
		position: sticky;
		top: 0;
	}

	th:hover {
		cursor: pointer;
	}
</style>
