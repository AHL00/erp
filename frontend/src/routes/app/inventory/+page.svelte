<script lang="ts">
	import MaxSizeSkeleton from '$lib/../components/MaxSizeSkeleton.svelte';
	import EditItemInline from './EditItemInline.svelte';
	import { type InventoryItemListRequest } from '$bindings/InventoryItemListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import { api_call } from '$lib/backend';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import type { SortOrder } from '$bindings/SortOrder';
	import SideBar from '../../../components/SideBar.svelte';
	import EditItemPanel from './EditItem.svelte';
	import type { InventoryField } from './field';

	let current_item_list_req: InventoryItemListRequest = {
		range: {
			count: 100,
			offset: 0
		},
		filters: [
			{
				column: 'quantity_per_box',
				operator: '<=',
				value: { Int: 12 }
			}
		],
		sorts: []
	};

	let current_item_list: InventoryItem[] = [];

	let refresh_loading_count = 0;

	function refresh_item_list() {
		refresh_loading_count++;
		api_call('inventory/list', 'POST', current_item_list_req).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					current_item_list = data;
				});
			} else {
				console.log('Error fetching item list');
				// This will clear the list, which will present a loading element of some kind
				current_item_list = [];
			}

			refresh_loading_count--;
		});
	}

	refresh_item_list();

	interface TableColumn {
		db_name: string;
		label: string;
		current_sort: SortOrder | null;
	}

	let columns: TableColumn[] = [
		{
			db_name: 'id',
			label: 'ID',
			current_sort: null
		},
		{
			db_name: 'name',
			label: 'Name',
			current_sort: null
		},
		{
			db_name: 'price',
			label: 'Price',
			current_sort: null
		},
		{
			db_name: 'stock',
			label: 'Stock',
			current_sort: null
		},
		{
			db_name: 'quantity_per_box',
			label: 'Qty/Box',
			current_sort: null
		}
	];

	function sort_toggle(column_index: number) {
		let column = columns[column_index];

		if (column.current_sort == null || column.current_sort == 'ASC') {
			// Update the current item list request
			let order: SortOrder = column.current_sort == null ? 'ASC' : 'DESC';

			let new_sort = {
				column: column.db_name,
				order: order
			};

			// If exists, replace. If not, push.
			let existing_sort_index = current_item_list_req.sorts.findIndex(
				(sort) => sort.column == column.db_name
			);
			if (existing_sort_index != -1) {
				current_item_list_req.sorts[existing_sort_index] = new_sort;
			} else {
				current_item_list_req.sorts.push(new_sort);
			}

			column.current_sort = order;
		} else if (column.current_sort == 'DESC') {
			column.current_sort = null;
			current_item_list_req.sorts = current_item_list_req.sorts.filter(
				(sort) => sort.column != column.db_name
			);
		}

		/// Hack so svelte reactivity works
		columns = columns;

		refresh_item_list();
	}

	let sidebar: SideBar;
	let edit_panel: EditItemPanel;

	$: {
		if (refresh_loading_count > 0) {
			document.getElementById('inv_table_body')?.classList.add('blur-md');
		} else {
			document.getElementById('inv_table_body')?.classList.remove('blur-md');
		}
	}

	let edit_callback = (item_id: number, edited_columns: InventoryField[]) => {
		// If sort or filter includes the edit column, we need to refresh the list
        let refresh_needed = false;
        
        for (let column of columns) {
            if (current_item_list_req.sorts.findIndex((sort) => sort.column == column.db_name) != -1) {
                refresh_needed = true;
                break;
            }

            if (current_item_list_req.filters.findIndex((filter) => filter.column == column.db_name) != -1) {
                refresh_needed = true;
                break;
            }
        }

        if (refresh_needed) {
            refresh_item_list();
            return;
        }

		api_call(`inventory/${item_id}`, 'GET', null).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					let item = data as InventoryItem;
					let index = current_item_list.findIndex((item) => item.id == item_id);
					current_item_list[index] = item;
					current_item_list = current_item_list;
				});
			}
		});
	};

	let edit_all_mode = false;
</script>

<PermissionGuard permissions={['INVENTORY_READ']}>
	<SideBar
		bind:this={sidebar}
		width="400px"
		close_callback={() => {
			edit_panel.clear_current();
		}}
	>
		<!-- Sidebar -->
		<div slot="sidebar" class="h-full w-full">
			<EditItemPanel
				bind:this={edit_panel}
				close_callback={() => {
					sidebar.close_sidebar();
				}}
				edit_callback={edit_callback}
			/>
		</div>
		<!-- Content Wrapper -->
		<div class="h-full w-full flex flex-col justify-start overflow-hidden" slot="content">
			<div class="flex-none">
				<span class="text-2xl">Inventory</span>
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
								<PermissionGuard permissions={['INVENTORY_WRITE']}>
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
										{column.label}
										{#if column.current_sort == 'ASC'}
											<span>▲</span>
										{/if}
										{#if column.current_sort == 'DESC'}
											<span>▼</span>
										{/if}

										{#if column.current_sort != null}
											<span
												>{current_item_list_req.sorts.findIndex(
													(sort) => sort.column == column.db_name
												) + 1}</span
											>
										{/if}
									</th>
								{/each}
								<PermissionGuard permissions={['INVENTORY_WRITE']}>
									{#if edit_all_mode}
										<th class="p-2 z-20 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-t-2xl">
										</th>
									{/if}
								</PermissionGuard>
							</tr>
						</thead>
						<tbody id="inv_table_body">
							{#if !edit_all_mode}
								{#each current_item_list as item}
									<tr class="text-center">
										<PermissionGuard permissions={['INVENTORY_WRITE']}>
											<td class="p-2">
												<button
													class="font-bold"
													on:click={() => {
														edit_panel.edit_item(item.id);
														sidebar.open_sidebar();
													}}
												>
													<i class="fa-solid fa-pen-to-square ml-2 opacity-80"></i>
												</button>
											</td>
										</PermissionGuard>
										<td class="p-2">{item.id}</td>
										<td class="p-2">{item.name}</td>
										<td class="p-2">{item.price}</td>
										<td class="p-2">{item.stock}</td>
										<td class="p-2">{item.quantity_per_box}</td>
									</tr>
								{/each}
							{:else}
								<PermissionGuard permissions={['INVENTORY_WRITE']}>
									{#each current_item_list as item}
										<EditItemInline {item} />
									{/each}
								</PermissionGuard>
							{/if}
						</tbody>
					</table>
				</div>

				<!-- Table controls -->
				<div class="inline-flex px-2 pt-3 pb-2">
					<PermissionGuard permissions={['INVENTORY_WRITE']}>
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
