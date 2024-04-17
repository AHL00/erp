<script lang="ts" generics="EditObject extends { id: number }">
	import { match } from 'ts-pattern';

	import type { CrudColumn, CrudEditTypeNumber } from './types';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import Loader from '../Loader.svelte';
	import type { CrudEditTypeString } from '$lib/../components/crud/types';
	import { read } from '$app/server';
	let current_item_id: number | null = null;

	/// Object with an 'id' field that can be retrieved from the backend
	export let current_editing_item: EditObject | null;

	export let columns: CrudColumn[];

	export let api_endpoint: string;

	let loading_counter = 0;

	export function edit(item_id: number) {
		current_item_id = item_id;

		loading_counter++;

		// Fetch the item from the backend
		api_call(`${api_endpoint}/${item_id}`, 'GET', null).then(
			(res) => {
				if (res?.status == 200) {
					res.json().then((data) => {
						current_editing_item = data;

						// refresh_form_values();
					});
				} else {
					// TODO: test this
					console.error('Error fetching item');
					toast.push('Error fetching item to edit');

					end_and_close();
				}
				loading_counter--;
			},
			(err) => {
				console.error(err);
				toast.push('Error fetching item to edit');
				end_and_close();
			}
		);
	}

	export function submit_form() {
		let form = document.getElementById(`edit-${api_endpoint}-form`) as HTMLFormElement;

		let form_data = new FormData(form);

		let data: Record<string, any> = {};

		for (let [key, value] of form_data.entries()) {
			data[key] = value;
		}

		let edited_columns: CrudColumn[] = [];

		let request: any = {};
		let malformed = false;

		for (let column of columns) {
			if (column.edit_type.type == 'hidden') continue;

			let value = data[column.api_name];

			if (column.edit_type.type == 'number') {
				let num = column.edit_type.data.integer
					? parseInt(value as string)
					: parseFloat(value as string);

				if (isNaN(num)) {
					toast.push(`${column.display_name} must be a number`);
					return;
				}

				if (column.edit_type.data.integer && !Number.isInteger(num)) {
					toast.push(`${column.display_name} must be a whole number`);
					return;
				}

				if (column.edit_type.data.range[1] != null && num > column.edit_type.data.range[1]) {
					toast.push(
						`${column.display_name} must be less than or equal to ${column.edit_type.data.range[1]}`
					);
					return;
				}

				if (column.edit_type.data.range[0] != null && num < column.edit_type.data.range[0]) {
					toast.push(
						`${column.display_name} must be greater than or equal to ${column.edit_type.data.range[0]}`
					);
					return;
				}

				if (num % column.edit_type.data.step != 0) {
					toast.push(`${column.display_name} must be a multiple of ${column.edit_type.data.step}`);
					return;
				}

				request[column.api_name] = num;
			} else if (column.edit_type.type == 'string') {
				let value_str = value as string;

				if (value_str.length < column.edit_type.data.length_range[0]) {
					toast.push(
						`${column.display_name} must be at least ${column.edit_type.data.length_range[0]} characters long`
					);
					return;
				}

				if (
					column.edit_type.data.length_range[1] != null &&
					value_str.length > column.edit_type.data.length_range[1]
				) {
					toast.push(
						`${column.display_name} must be at most ${column.edit_type.data.length_range[1]} characters long`
					);
					return;
				}

				if (
					column.edit_type.data.regex != null &&
					!new RegExp(column.edit_type.data.regex).test(value_str)
				) {
					toast.push(
						`${column.display_name} must match the pattern ${column.edit_type.data.regex}`
					);
					return;
				}

				request[column.api_name] = value_str;
			}

			edited_columns.push(column);
		}

		if (malformed) {
			toast.push('Error submitting form');
			return;
		}

		api_call(`${api_endpoint}/${current_item_id}`, 'PATCH', request).then((res) => {
			if (res?.status == 200) {
				toast.push('Item edited successfully');
				edited_callback(current_item_id!, edited_columns);
				end_and_close();
			} else {
				toast.push('Error editing item');
			}
		});
	}

	export function clear_current() {
		current_item_id = null;
		current_editing_item = null;
	}

	export let close_callback: () => void;

	export let edited_callback: (item_id: number, edited_columns: CrudColumn[]) => void;

	function end_and_close() {
		close_callback();
		clear_current();
	}

	function get_column_value(api_name: string): any {
		if (current_editing_item == null) return null;

		return (current_editing_item as any)[api_name];
	}
</script>

<div class="flex flex-col h-full w-full items-center">
	<div class="my-4">
		<span class="text-2xl">Edit Item</span>
	</div>

	<form class="flex-grow flex-col space-y-4" id="edit-{api_endpoint}-form">
		{#if loading_counter > 0 || current_editing_item == null}
			{#if loading_counter == 0 && current_editing_item == null}
				<div class="flex-grow">Error loading item</div>
			{:else}
				<Loader />
			{/if}
		{:else}
			<!-- Here, the item data should be populated in the variable -->
			<!-- TODO: Reset buttons beside each field -->

			{#each columns as column, j}
				{#if column.edit_type.type != 'hidden'}
					<div class="flex flex-row space-x-2">
						<label for={column.api_name}>{column.display_name}</label>
						{#if column.edit_type.type == 'number'}
							<input
								id="{api_endpoint}-{column.api_name}-input"
								form="edit-{api_endpoint}-form"
								class="flex-grow"
								type="number"
								name={column.api_name}
								value={// @ts-ignore
								get_column_value(column.api_name)}
								min={column.edit_type.data.range[0]}
								max={column.edit_type.data.range[1]}
								step={column.edit_type.data.step}
							/>
						{:else if column.edit_type.type == 'string'}
							<input
								id="{api_endpoint}-{column.api_name}-input"
								form="edit-{api_endpoint}-form"
								class="flex-grow"
								type="text"
								name={column.api_name}
								value={// @ts-ignore
								get_column_value(column.api_name)}
								minlength={column.edit_type.data.length_range[0]}
								maxlength={column.edit_type.data.length_range[1]}
								pattern={column.edit_type.data.regex}
								required={column.edit_type.data.length_range[0] > 0}
							/>
						{:else if column.edit_type.type == 'checkbox'}
							<input
								id="{api_endpoint}-{column.api_name}-input"
								form="edit-{api_endpoint}-form"
								type="checkbox"
								name={column.api_name}
								checked={// @ts-ignore
								get_column_value(column.api_name)}
							/>
						{:else if column.edit_type.type == 'select'}
							<!-- TODO: Make this work??? Is it even needed -->
							<select>
								{#each column.edit_type.data.options as option}
									<option
										value={option.value}
										selected={option.value == get_column_value(column.api_name)}
										>{option.display}</option
									>
								{/each}
							</select>
						{/if}
						<button
							on:click={() =>
								// @ts-ignore
								(document.getElementById(`${api_endpoint}-${column.api_name}-input`).value =
									get_column_value(column.api_name))}
						>
							<i class="fa-solid fa-rotate-left"></i>
						</button>
					</div>
				{/if}
			{/each}
		{/if}

		<div class="self-end mt-auto">
			<button on:click={end_and_close}>Cancel</button>
			{#if !(loading_counter > 0 || current_editing_item == null)}
				<button
					type="submit"
					form="edit-{api_endpoint}-form"
					on:click={submit_form}
					on:submit={(e) => {
						e.preventDefault();
					}}>Save</button
				>
			{/if}
		</div>
	</form>
</div>
