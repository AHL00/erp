<script lang="ts" generics="EditObject extends { id: number }">
	import type { CrudColumn } from './types';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';

    let current_item_id: number | null = null;

    /// Object with an 'id' field that can be retrieved from the backend
	export let current_editing_item: EditObject | null;

    export let api_endpoint: string;

	let loading_counter = 0;

	export function edit_item(item_id: number) {
		current_item_id = item_id;

		loading_counter++;

		// Fetch the item from the backend
		api_call(`${api_endpoint}/${item_id}`, 'GET', null).then((res) => {
			if (res?.status == 200) {
				res.json().then((data) => {
					current_editing_item = data;
				});
			} else {
                // TODO: test this
                console.error('Error fetching item');
                toast.push('Error fetching item to edit');

                close();
			}
			loading_counter--;
		});
	}

	export function clear_current() {
		current_item_id = null;
		current_editing_item = null;
	}

	export let close_callback: () => void;   

    export let edited_callback: (item_id: number, edited_columns: CrudColumn[]) => void;

    function close() {
		close_callback();
		clear_current();
	};
</script> 