<!-- A generic CRUD page that can edit any specified object on the API. -->
<script lang="ts" generics="EntryType extends { id: number }">
	import { flip } from 'svelte/animate';

	import type { SearchRequest } from '$bindings/SearchRequest';

	import CurrencySpan from '../currency/CurrencySpan.svelte';

	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';
	import PermissionGuard from '../PermissionGuard.svelte';
	import type { CrudColumn } from './types';
	import type { ListSort } from '$bindings/ListSort';
	import type { ListFilter } from '$bindings/ListFilter';
	import SideBar from '../SideBar.svelte';
	import CrudEditPanel from './CrudEditPanel.svelte';
	import type { InventoryItem } from '$bindings/InventoryItem';
	import type { ListRequest } from '$bindings/ListRequest';
	import Loader from '$lib/../components/Loader.svelte';
	import { onMount } from 'svelte';
	import { format_local_date, utc_date_to_local_rounded } from '$lib';
	import { fade, fly, scale } from 'svelte/transition';

	// List request type, e.g. InventoryItemListRequest
	export let list_request: ListRequest;
	// Returned type from list, e.g. InventoryItem
	export let objects_list: EntryType[];
	export let crud_endpoint: string;

	type CustomButton = {
		text: string;
		callback: (entry: EntryType) => void;
		font_awesome_icon: string;
		permissions: UserPermissionEnum[];
	};

	export let custom_buttons: CustomButton[] = [];

	export let custom_margins: string = 'm-0';
	export let post_delete_callback: (res: Response) => void = () => {};

	export let read_perms: [UserPermissionEnum];
	export let create_perms: [UserPermissionEnum];
	export let update_perms: [UserPermissionEnum];
	export let delete_perms: [UserPermissionEnum];

	/// Allows the parent to override the default edit function. It will send the item that was edited as an argument.
	export let edit_override: ((item_id: number) => void) | null = null;

	/// The default item to create when the create button is pressed.
	/// The ID field will be ignored, and the rest of the fields will be used to create the item.
	/// If null, the create button will be disabled.
	export let create_post_request: any | null;

	/// Tailwind classes for the background color
	/// Both light and dark mode are needed.
	/// Default = 'bg-custom-light dark:bg-custom-dark'
	export let background_color: string = 'bg-custom-lighter dark:bg-custom-dark';

	let loading_count = 0;

	let error = false;

	let search_request: SearchRequest | null = null;
	let last_search: string | null = null;
	let search_timer: number | null = null;
	let search_input: HTMLInputElement;
	const search_count: number = 1000000;

	function get_api_request_name(column: CrudColumn): string {
		return column.api_request_name ?? column.api_name;
	}

	function get_column_by_api_name(api_name: string): CrudColumn | undefined {
		return columns.find((column) => get_api_request_name(column) === api_name);
	}

	function get_column_by_search_column_name(col_name: string): CrudColumn | undefined {
		console.log(col_name);
		return columns.find((column) => {
			if (!column.search_nested) {
				return get_api_request_name(column) === col_name;
			} else {
				return column.search_nested === col_name;
			}
		});
	}

	function refresh_list() {
		loading_count++;

		// If search_request exists, search instead
		if (search_request !== null) {
			api_call(`${crud_endpoint}/search`, 'POST', search_request)
				.then((res) => {
					if (res === undefined) {
						console.error('Error fetching list');

						toast.push('Error fetching data');

						// Clear the list
						objects_list = [];
						last_search = null;

						loading_count--;
						error = true;
						return;
					}

					if (res?.status == 200) {
						res.json().then((data) => {
							objects_list = data;
							last_search = search_request ? search_request.search : null;
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
						last_search = null;

						loading_count--;
						error = true;
					}
				})
				.catch((e) => {
					console.error('Error fetching list');

					toast.push('Error fetching data');

					// Clear the list
					objects_list = [];
					last_search = null;

					loading_count--;
					error = true;
				});

			return;
		}

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
					last_search = null;

					loading_count--;
					error = true;
				}
			})
			.catch((e) => {
				console.error('Error fetching list');

				toast.push('Error fetching data');

				// Clear the list
				objects_list = [];
				last_search = null;

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

	let edited_callback = (item_id: number, edited_columns: CrudColumn[]) => {
		// If sort or filter includes the edit column, we need to refresh the list
		let refresh_needed = false;

		for (let column of edited_columns) {
			if (
				list_request.sorts.findIndex(
					(sort: ListSort) => sort.column == get_api_request_name(column)
				) != -1
			) {
				refresh_needed = true;
				break;
			}

			if (
				list_request.filters.findIndex(
					(filter: ListFilter) => filter.column == get_api_request_name(column)
				) != -1
			) {
				refresh_needed = true;
				break;
			}
		}

		if (refresh_needed) {
			refresh_list();
			return;
		}

		// Update the item in the list
		api_call(`${crud_endpoint}/${item_id}`, 'GET', null)
			.then((res) => {
				if (res?.status == 200) {
					res?.json().then((data) => {
						let index = objects_list.findIndex((item: any) => item.id == item_id);
						objects_list[index] = data;

						// Hack for svelte reactivity
						objects_list = objects_list;
					});
				} else {
					console.error('Error fetching item, refreshing whole list as fallback');

					// Refresh the list
					refresh_list();
				}
			})
			.catch((e) => {
				console.error('Error fetching item, refreshing whole list as fallback');

				// Refresh the list
				refresh_list();
			});
	};

	let edit_all_mode = false;

	let sidebar: SideBar;
	let edit_panel: CrudEditPanel<InventoryItem>;
	let edit_item: InventoryItem;

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
			last_search = null;

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
			last_search = null;

			loading_count--;
			return Promise.reject('Error fetching count: HTTP code ' + res?.status);
		}
	}

	function create_new_handler() {
		// This should not be called if the create button is disabled, but just in case
		if (create_post_request == null) {
			return;
		}

		if (create_post_request != null) {
			// Try to send an API call to create the item
			try {
				api_call(`${crud_endpoint}`, 'POST', create_post_request).then((res) => {
					if (res === undefined) {
						console.error('Error creating item: no response');
						toast.push('Error creating item: no response');
						return;
					}

					if (res?.status == 201) {
						// The server should return an id of the created item
						res?.json().then((id) => {
							console.log('Created an item with the id: ', id);

							// Refresh the list
							refresh_list();

							// Update page count
							get_page_count().then((n) => {
								page_count = n;
							});

							get_item_from_api(id)
								.then((item) => {
									edit_item_handler(item);
								})
								.catch((e) => {
									console.error('Error fetching created item: ', e);
									toast.push('Error fetching created item');
								});
						});
					} else {
						console.error('Error creating item: HTTP code ', res?.status);
						toast.push(`Error creating item: HTTP code ${res?.status}`);
					}
				});
			} catch (e) {
				console.error('Error creating item: ', e);
				toast.push('Error creating item');
			}
		} else {
			console.error(
				'No default item provided for creation, create should be disabled or a default item should be provided'
			);
		}
	}

	function get_item_in_list(id: number): EntryType | undefined {
		return objects_list.find((item) => item.id == id);
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

	function edit_item_handler(item: EntryType) {
		// If the parent has specified an edit function, use that instead of the default edit panel
		if (edit_override != null) {
			edit_override(item.id);
			return;
		}

		// @ts-ignore
		edit_panel.edit(item.id, item);
		sidebar.open_sidebar();
	}

	function delete_item_handler(item: EntryType) {
		if (confirm('Are you sure you want to delete this item? This action is irreversible.')) {
			api_call(`${crud_endpoint}/${item.id}`, 'DELETE', null)
				.then((res) => {
					if (res?.status == 204 || res?.status == 200) {
						toast.push('Item deleted');
						refresh_list();
						post_delete_callback(res);
					} else {
						toast.push('Error deleting item');
					}
				})
				.catch((e) => {
					toast.push('Error deleting item');
				});
		}
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

	export let delete_enabled: boolean = false;

	function verify_float(value: any): number | null {
		if (typeof value === 'number') {
			return value;
		}

		if (typeof value === 'string') {
			try {
				return parseFloat(value);
			} catch {
				return null;
			}
		}
		return null;
	}

	$: {
		if (loading_count == 0 && search_request !== null) {
			if (search_input && search_input !== document.activeElement)
				// To solve weird thing where
				// rerendering causes blurring.
				setTimeout(() => {
					search_input.focus();
				}, 100);
		}
	}
</script>

<!-- TODO: Allow custom edit panel, make it fit in any space with flex-grow -->

<PermissionGuard permissions={read_perms}>
	<SideBar
		bind:this={sidebar}
		width="400px"
		close_callback={() => {
			edit_panel.clear_current();
		}}
	>
		<!-- Sidebar -->
		<div slot="sidebar" class="h-full w-full">
			<!-- bind:this={edit_panel} -->
			<CrudEditPanel
				api_endpoint={crud_endpoint}
				bind:this={edit_panel}
				current_editing_item={edit_item}
				{columns}
				close_callback={() => {
					sidebar.close_sidebar();
				}}
				{edited_callback}
			/>
		</div>
		<!-- Content Wrapper -->
		<div
			slot="content"
			class="relative flex flex-col flex-grow self-center w-full h-full
            {background_color}
            "
		>
			{#if search_request}
				<div
					class="searchbar p-2 flex flex-row justify-center place-items-center gap-x-2"
					in:fly={{ y: -200, duration: 300 }}
					out:fly={{ y: -200, duration: 300 }}
				>
					<div class="relative w-1/2">
						<input
							type="text"
							class="w-full h-full border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded-md p-2 bg-transparent relative z-auto pr-10"
							bind:value={search_request.search}
							disabled={loading_count > 0}
							autocomplete="off"
							autocapitalize="off"
							placeholder={`Search by "${get_column_by_search_column_name(search_request.column)?.display_name}"...`}
							on:keydown={(e) => {
								if (e.key === 'Enter') {
									refresh_list();
								} else if (e.key === 'Escape') {
									search_request = null;
									last_search = null;
									refresh_list();
								}
							}}
							bind:this={search_input}
						/>
						{#if last_search == null || last_search !== search_request.search}
							<button
								class="fas fa-search absolute right-10 top-1/2 transform -translate-y-1/2
                            h-full hover:brightness-90 pl-3 pr-2
                            border-l-[1px] border-custom-light-outline dark:border-custom-dark-outline"
								transition:fade={{ duration: 200 }}
								on:click={() => {
									refresh_list();
								}}
							>
							</button>
						{/if}
						<button
							class="fas fa-times absolute right-0 top-1/2 transform -translate-y-1/2
                            h-full hover:brightness-90 px-3 text-lg
                            border-l-[1px] border-custom-light-outline dark:border-custom-dark-outline"
							on:click={() => {
								search_request = null;
								last_search = null;
								refresh_list();
							}}
						>
						</button>
					</div>
				</div>
			{/if}
			<div class="overflow-auto h-full flex flex-col {custom_margins}">
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
				<table class="w-full">
					<thead>
						<tr>
							<!-- NOTE: This background color will have to be changed if the body's background color is changed.  
                            Inheriting is a mess so I'm just going to hardcode it. -->
							<PermissionGuard permissions={update_perms}>
								{#if !edit_all_mode}
									<th class="p-2 z-20 {background_color}"> Edit </th>
								{/if}
							</PermissionGuard>
							<PermissionGuard permissions={delete_perms}>
								{#if delete_enabled}
									<th class="p-2 z-20 {background_color}"> Delete </th>
								{/if}
							</PermissionGuard>
							{#each custom_buttons as button}
								<PermissionGuard permissions={button.permissions}>
									<th class="p-2 z-20 {background_color}"> {button.text} </th>
								</PermissionGuard>
							{/each}
							{#each columns as column, index}
								<th
									class="p-2 z-20 {background_color} text-{column.align} text-left"
									on:click={() => {
										if (search_request === null) {
											sort_toggle(index);
										}
									}}
								>
									<span style="white-space: nowrap;">
										{column.display_name}
										{#if search_request === null}
											{#if column.current_sort == 'ASC'}
												<span class="text-xl">▲</span>
											{/if}
											{#if column.current_sort == 'DESC'}
												<span class="text-xl">▼</span>
											{/if}
											{#if column.current_sort != null && list_request.sorts.length > 1}
												<span class="text-xs"
													>{find_sort_index(get_api_request_name(column)) + 1}</span
												>
											{/if}
										{/if}
										{#if column.searchable && search_request === null}
											<button
												class="
                                                flex-row gap-x-1
                                                inline-flex items-center
                                                rounded-lg h-7 px-2 dark:bg-custom-dark hover:brightness-90
                                                outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline"
												on:click={() => {
													search_request = {
														column: column.search_nested ?? column.api_name,
														count: search_count,
														search: '',
														nested_access: column.search_nested ? column.search_nested : null
													};
													objects_list = [];
													last_search = null;

													// Focus on the search input
													setTimeout(() => {
														search_input.focus();
													}, 100);
												}}
											>
												<i class="fas text-sm mr-1 fa-search"></i>
												<span class="text-sm">Search</span>
											</button>
										{/if}
									</span>
								</th>
							{/each}
							<!-- <PermissionGuard permissions={write_perms}>
								{#if edit_all_mode}
									<th class="p-2 z-20 {background_color}"> </th>
								{/if}
                                {#if !delete_enabled}
                                    <th class="p-2 z-20 {background_color}"> </th>
                                {/if}
							</PermissionGuard> -->
						</tr>
					</thead>
					<tbody id="{crud_endpoint}_table_body">
						{#if !edit_all_mode}
							{#each objects_list as item}
								<tr>
									<PermissionGuard permissions={update_perms}>
										{#if !edit_all_mode}
											<td class="p-2 text-center">
												<button
													class="font-bold"
													on:click={() => {
														edit_item_handler(item);
													}}
												>
													<i class="fa-solid fa-pen-to-square ml-2 opacity-80"></i>
												</button>
											</td>
										{/if}
									</PermissionGuard>
									<PermissionGuard permissions={delete_perms}>
										{#if delete_enabled}
											<td class="p-2 text-center">
												<button
													class="font-bold"
													on:click={() => {
														delete_item_handler(item);
													}}
												>
													<i class="fa fa-trash ml-2 opacity-80"></i>
												</button>
											</td>
										{/if}
									</PermissionGuard>
									{#each custom_buttons as button}
										<PermissionGuard permissions={button.permissions}>
											<td class="p-2 text-center">
												<button
													class="font-bold"
													on:click={() => {
														button.callback(item);
													}}
												>
													<i class="{button.font_awesome_icon} ml-2 opacity-80"></i>
												</button>
											</td>
										</PermissionGuard>
									{/each}
									{#each columns as column}
										<td class="p-2 text-{column.align}">
											{#if column.display_map_fn !== null}
												{column.display_map_fn(get_field_of_item(item, column.api_name))}
											{:else if column.type.type == 'currency'}
												{@const currency_value = verify_float(
													get_field_of_item(item, column.api_name)
												)}
												{#if currency_value === null}
													<span class="text-red-500">[Invalid currency]</span>
												{:else}
													<CurrencySpan value={currency_value} align={column.align} />
												{/if}
											{:else if column.type.type == 'datetime'}
												{format_local_date(
													new Date(
														utc_date_to_local_rounded(
															get_field_of_item(item, column.api_name),
															column.type.accuracy
														)
													),
													column.type.format
												)}
											{:else if column.type.type == 'checkbox'}
												<input
													type="checkbox"
													checked={get_field_of_item(item, column.api_name)}
													disabled
												/>
											{:else}
												{get_field_of_item(item, column.api_name)}
											{/if}
										</td>
									{/each}
								</tr>
							{/each}
						{:else}
							<PermissionGuard permissions={delete_perms}>
								{#each objects_list as item}
									{#if delete_enabled}
										<td class="p-2">
											<button
												class="font-bold"
												on:click={() => {
													delete_item_handler(item);
												}}
											>
												<i class="fa fa-trash ml-2 opacity-80"></i>
											</button>
										</td>
									{/if}
									<!-- <EditItemInline {item} /> -->
								{/each}
							</PermissionGuard>
						{/if}
					</tbody>
				</table>
			</div>

			<!-- Table controls -->
			<div class="inline-flex px-2 pt-3 pb-2 justify-start">
				<PermissionGuard permissions={update_perms}>
					{#if edit_override === null && false}
						<div class="mx-1">
							<button
								type="button"
								class="rounded-lg outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline
                            text-custom-text-light-lighter dark:text-custom-text-dark-lighter px-2 dark:bg-custom-dark hover:brightness-90
                            inline-flex items-center h-7"
								on:click={() => {
									edit_all_mode = !edit_all_mode;
								}}
							>
								{#if edit_all_mode}
									<span class="text-md"> Done </span>
									<i class="fa-solid fa-check ml-2"></i>
								{:else}
									<span class="text-md"> Edit all </span>
									<i class="fa-solid fa-pen-to-square ml-2"></i>
								{/if}
							</button>
						</div>
					{/if}
				</PermissionGuard>

				<div class="mx-1">
					<div
						class="inline-flex rounded-md text-custom-text-light-lighter dark:text-custom-text-dark-lighter gap-x-2"
						role="group"
					>
						<PermissionGuard permissions={create_perms}>
							{#if create_post_request !== null}
								<button
									type="button"
									class="rounded-lg h-7 px-2 dark:bg-custom-dark hover:brightness-90 inline-flex items-center
                            outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline z-10"
									on:click={() => {
										create_new_handler();
									}}
								>
									<i class="fas text-sm mr-2 fa-plus"></i>
									<span class="text-sm">New</span>
								</button>
							{/if}
						</PermissionGuard>

						<button
							type="button"
							class="rounded-lg h-7 px-2 dark:bg-custom-dark hover:brightness-90 inline-flex items-center
                                outline outline-1 outline-custom-light-outline dark:outline-custom-dark-outline z-10"
							on:click={() => {
								refresh_list();
							}}
						>
							<i class="fas text-sm mr-2 fa-sync"></i>
							<span class="text-sm">Refresh</span>
						</button>
					</div>
				</div>

				<div class="justify-self-end ml-auto mx-1 {search_request ? 'hidden' : ''}">
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
	</SideBar>
</PermissionGuard>

<style>
	th {
		position: sticky;
		top: 0;
	}

	th:hover {
		cursor: pointer;
	}

	.text-left {
		text-align: left !important;
	}

	.text-center {
		text-align: center !important;
	}

	.text-right {
		text-align: right !important;
	}
</style>
