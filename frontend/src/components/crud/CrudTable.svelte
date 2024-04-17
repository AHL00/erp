<!-- A generic CRUD page that can edit any specified object on the API. -->
<script lang="ts" generics="ListObject extends { id: number }">
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

	export let page_title: string;

	// List request type, e.g. InventoryItemListRequest
	export let list_request: ListRequest;
	// Returned type from list, e.g. InventoryItem
	export let objects_list: ListObject[];
	export let crud_endpoint: string;

	export let read_perms: [UserPermissionEnum];
	export let write_perms: [UserPermissionEnum];

	let refresh_loading_count = 0;

	function refresh_list() {
		refresh_loading_count++;

		api_call(`${crud_endpoint}/list`, 'POST', list_request).then((res) => {
			if (res?.status == 200) {
				res.json().then((data) => {
					objects_list = data;
				});
			} else {
				console.error('Error fetching list');

				// TODO: Proper error display
				toast.push('Error fetching data', {});

				// Clear the list, which will show a loading element
				objects_list = [];
			}
			refresh_loading_count--;
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
			(sort: ListSort) => sort.column == column.api_name
		);

		// Remove the sort if it exists
		if (sort_list_index != -1) {
			list_request.sorts.splice(sort_list_index, 1);
		}

		// Add the sort
		if (column.current_sort != null) {
			let sort: ListSort = {
				column: column.api_name,
				order: column.current_sort
			};

			list_request.sorts.push(sort);
		}

		// Hack svelte reactivity
		columns = columns;

		// Refresh the list
		refresh_list();
	}

	$: {
		if (refresh_loading_count > 0) {
			document.getElementById(`${crud_endpoint}_table_body`)?.classList.add('blur-md');
		} else {
			document.getElementById(`${crud_endpoint}_table_body`)?.classList.remove('blur-md');
		}
	}

	let edited_callback = (item_id: number, edited_columns: CrudColumn[]) => {
		// If sort or filter includes the edit column, we need to refresh the list
		let refresh_needed = false;

		for (let column of edited_columns) {
			if (list_request.sorts.findIndex((sort: ListSort) => sort.column == column.api_name) != -1) {
				refresh_needed = true;
				break;
			}

			if (
				list_request.filters.findIndex((filter: ListFilter) => filter.column == column.api_name) !=
				-1
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
		api_call(`inventory/${item_id}`, 'GET', null).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					let index = objects_list.findIndex((item: any) => item.id == item_id);
					objects_list[index] = data;

					// Hack for svelte reactivity
					objects_list = objects_list;
				});
			}
		});
	};

	let edit_all_mode = false;

	let sidebar: SideBar;
	let edit_panel: CrudEditPanel<InventoryItem>;
    let edit_item: InventoryItem | null = null;

	function find_sort_index(column_api_name: string): number {
		return list_request.sorts.findIndex((sort: ListSort) => sort.column == column_api_name);
	}

	function get_item_field(item: ListObject, field: string): string {
		return (item as any)[field];
	}
</script>

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
                api_endpoint="inventory"
                bind:this={edit_panel}
                current_editing_item={edit_item}
                columns={columns}
				close_callback={() => {
					sidebar.close_sidebar();
				}}
				{edited_callback}
			/>
		</div>
		<!-- Content Wrapper -->
		<div class="h-full w-full flex flex-col justify-start overflow-hidden" slot="content">
			<div class="flex-none">
				<span class="text-2xl">{page_title}</span>
			</div>

			<div
				class="flex flex-col flex-grow self-center w-fit min-w-[50%] h-0 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-2xl shadow-sm shadow-custom-bg-light-shadow dark:shadow-custom-bg-dark-shadow my-4"
			>
				<div class="overflow-auto h-full flex flex-col mt-3 mx-3">
					<table class="w-full">
						<thead>
							<tr>
								<!-- NOTE: This background color will have to be changed if the body's background color is changed.  
                            Inheriting is a mess so I'm just going to hardcode it. -->
								<PermissionGuard permissions={write_perms}>
									{#if !edit_all_mode}
										<th class="p-2 z-20 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-t-2xl">
											Edit
										</th>
									{/if}
								</PermissionGuard>
								{#each columns as column, index}
									<th
										class="p-2 z-20 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-t-2xl"
										on:click={() => {
											sort_toggle(index);
										}}
									>
										{column.display_name}
										{#if column.current_sort == 'ASC'}
											<span>▲</span>
										{/if}
										{#if column.current_sort == 'DESC'}
											<span>▼</span>
										{/if}

										{#if column.current_sort != null}
											<span>
												{find_sort_index(column.api_name) + 1}
											</span>
										{/if}
									</th>
								{/each}
								<PermissionGuard permissions={write_perms}>
									{#if edit_all_mode}
										<th class="p-2 z-20 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-t-2xl">
										</th>
									{/if}
								</PermissionGuard>
							</tr>
						</thead>
						<tbody id="{crud_endpoint}_table_body">
							{#if !edit_all_mode}
								{#each objects_list as item}
									<tr class="text-center">
										<PermissionGuard permissions={write_perms}>
											<td class="p-2">
												<button
													class="font-bold"
													on:click={() => {
														edit_panel.edit(item.id);
														sidebar.open_sidebar();
													}}
												>
													<i class="fa-solid fa-pen-to-square ml-2 opacity-80"></i>
												</button>
											</td>
										</PermissionGuard>
										{#each columns as column}
											<td class="p-2">
												{get_item_field(item, column.api_name)}
											</td>
										{/each}
									</tr>
								{/each}
							{:else}
								<PermissionGuard permissions={write_perms}>
									{#each objects_list as item}
										<!-- <EditItemInline {item} /> -->
									{/each}
								</PermissionGuard>
							{/if}
						</tbody>
					</table>
				</div>

				<!-- Table controls -->
				<div class="inline-flex px-2 pt-3 pb-2">
					<PermissionGuard permissions={write_perms}>
						<div class="flex-none self-start">
							<button
								type="button"
								class="rounded-lg outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline
                            text-custom-text-light-lighter dark:text-custom-text-dark-lighter px-2 dark:bg-custom-bg-dark hover:brightness-90
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
					</PermissionGuard>
					<div class="flex-none self-end ml-auto">
						<div
							class="inline-flex rounded-md outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline
                        text-custom-text-light-lighter dark:text-custom-text-dark-lighter"
							role="group"
						>
							<button
								type="button"
								class="rounded-l-lg px-2 dark:bg-custom-bg-dark hover:brightness-90 inline-flex items-center
                            outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline z-10"
							>
								<i class="fas fa-chevron-left text-sm"></i>
							</button>
							<button type="button" class="px-5 dark:bg-custom-bg-dark inline-flex items-center">
								<span class="text-lg">1</span>
							</button>
							<button
								type="button"
								class="rounded-r-lg px-2 dark:bg-custom-bg-dark hover:brightness-90 inline-flex items-center
                            outline outline-1 outline-custom-bg-light-outline dark:outline-custom-bg-dark-outline z-10"
							>
								<i class="fas fa-chevron-right text-sm"></i>
							</button>
						</div>
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
</style>
