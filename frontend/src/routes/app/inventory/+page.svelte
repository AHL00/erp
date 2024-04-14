<script lang="ts">
	import { type InventoryItemListRequest } from '$bindings/InventoryItemListRequest';
	import { type InventoryItem } from '$bindings/InventoryItem';
	import { api_call } from '$lib/backend';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import type { SortOrder } from '$bindings/SortOrder';
	import SideBar from '../../../components/SideBar.svelte';

	let current_item_list_req: InventoryItemListRequest = {
		range: {
			count: 1000,
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

	let edit_add_panel: SideBar;

	$: {
		if (refresh_loading_count > 0) {
			document.getElementById('inv_table_body')?.classList.add('blur-md');
		} else {
			document.getElementById('inv_table_body')?.classList.remove('blur-md');
		}
	}
</script>

<PermissionGuard permissions={['INVENTORY_READ']}>
	<SideBar bind:this={edit_add_panel} width="300px">
		<!-- Sidebar -->
		<div slot="sidebar">Hello</div>

		<!-- Content Wrapper -->
		<div class="h-full w-full flex flex-col justify-start overflow-hidden" slot="content">
			<div class="flex-none">
				<span class="text-2xl">Inventory</span>
			</div>

			<div
				class="flex flex-col flex-grow self-center w-4/5 h-0 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-2xl shadow-sm shadow-custom-bg-light-shadow dark:shadow-custom-bg-dark-shadow my-4"
			>
				<div class="overflow-auto w-full mt-4">
					<table class="w-full">
						<thead>
							<tr>
								<!-- NOTE: This background color will have to be changed if the body's background color is changed.  
                            Inheriting is a mess so I'm just going to hardcode it. -->
								{#each columns as column, index}
									<th
										class="p-2 bg-custom-bg-lighter dark:bg-custom-bg-dark rounded-t-2xl"
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
							</tr>
						</thead>
						<tbody id="inv_table_body">
							{#each current_item_list as item}
								<tr class="text-center">
									<td>{item.id}</td>
									<td>{item.name}</td>
									<td>{item.price}</td>
									<td>{item.stock}</td>
									<td>{item.quantity_per_box}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<!-- Table controls -->
				<div class="flex-none self-end">
                    <button
                        class="bg-custom-bg-light dark:bg-custom-bg-dark text-white dark:text-black rounded-2xl p-2 m-2"
                        on:click={edit_add_panel.toggleSidebar}
                    >
                        Add/Edit
                    </button>
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

	td {
		padding: 0.5rem;
	}
</style>
