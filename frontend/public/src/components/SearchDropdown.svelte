<!--
 A component which allows the user to search for a value, and select one from a dropdown list. 
 The dropdown list is populated as the user types.
-->
<script lang="ts" generics="ResultType">
	import { auth_info_store } from '$lib/auth';

	import { api_call } from '$lib/backend';

	import Loader from './Loader.svelte';

	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';
	import type { SearchRequest } from '$bindings/SearchRequest';

	import { onDestroy, onMount } from 'svelte';

	export let input_id: string;
	export let input_placeholder: string;

	/// Milliseconds to wait after the user stops typing before sending the search request
	export let typing_search_delay: number = 500;

	// The endpoint to send the search request to.
	// It must be a POST request accepting the type SearchRequest as the body.
	export let search_endpoint: string;
	export let search_perms: UserPermissionEnum[] = [];
    /// NOTE: This is just a type indicator
	export let search_results: ResultType[] = [];
	export let search_column: string;
	export let search_count: number;
	export let display_map_fn: (val: any) => string;
	export let initial_value: ResultType | null = null;
	export let max_dropdown_height: string = '300px';
	export let form_id: string;
	export let required: boolean = false;
	export let validity_message: string = 'Select a value from the dropdown';
	export let on_change: (val: ResultType) => void = () => {};

	let search_input: HTMLInputElement;
	let dropdown_div: HTMLDivElement;

	let current_search = '';
	let loading = false;
	let search_error: string | null = null;

	let selected: ResultType | null = null;

	function select(idx: number) {
		selected = search_results[idx];

		// Set the search input to the selected value
		search_input.value = display_map_fn(selected);
		on_change(selected);

		close();
	}

	export function set_selected_value(value: ResultType) {
		selected = value;
		search_input.value = display_map_fn(selected);

		on_change(selected);
		close();
	}

	function close() {
		// If closed without selecting, reset the search input
		if (selected === null) {
			search_input.value = '';
		} else {
			search_input.value = display_map_fn(selected);
		}

		search_input.blur();
		dropdown_div.classList.add('hidden');
	}

	let typing_timer: any;

	function typing_handler() {
		// Clear the timer
		clearTimeout(typing_timer);

		// Set the timer
		typing_timer = setTimeout(() => {
			// If the search input is empty, don't search
			if (search_input.value === '') {
				search_results = [];
				return;
			}

			// If the search input is the same as the last search, don't search
			if (search_input.value === current_search) return;

			// Set the current search
			current_search = search_input.value;

			// Search
			search();
		}, typing_search_delay);
	}

	function search() {
		if (search_perms.length > 0) {
			let auth_info = $auth_info_store;

			if (auth_info === null || auth_info === undefined) {
				search_error = 'Not authenticated';
				return;
			}

			// TODO: Test permissions

			let user_perms = auth_info.permissions;

			if (user_perms.includes('ADMIN')) {
				// If it includes ADMIN, then it will be allowed
			} else if (!search_perms.some((perm) => auth_info.permissions.includes(perm))) {
				search_error = 'Permission denied';
				return;
			}
		}

		loading = true;
		search_error = null;

		let search_request: SearchRequest = {
			search: search_input.value,
			column: search_column,
			count: search_count
		};

		// Fetch the search results
		api_call(search_endpoint, 'POST', search_request)
			.then((res) => {
				if (res === undefined) {
					search_error = 'Error searching';
					return;
				}

				if (res?.status === 200) {
					res
						.json()
						.then((data) => {
							search_results = data;
							loading = false;
						})
						.catch((err) => {
							console.error(err);
							search_error = 'Error parsing response';
						});
				} else {
					search_error = 'Error searching';
				}
			})
			.catch((err) => {
				search_error = 'Error searching';
			});
	}

	let keydown_listener = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			if (dropdown_div !== null && dropdown_div !== undefined) {
				if (!dropdown_div.classList.contains('hidden')) {
					close();
				}
			}
		}
	};

	let click_listener = (e: MouseEvent) => {
		if (dropdown_div !== null && dropdown_div !== undefined) {
			if (!dropdown_div.contains(e.target as Node) && !search_input.contains(e.target as Node)) {
				close();
			}
		}
	};

	onMount(() => {
		if (required) {
			search_input.required = true;
		}

		search_input.setAttribute('form', form_id);
		// search_input.setCustomValidity(validity_message);

		// Escape handler if dropdown is open
		window.addEventListener('keydown', keydown_listener);

		// Click handler
		window.addEventListener('click', click_listener);

		if (initial_value !== null) {
			set_selected_value(initial_value);
		}
	});

	onDestroy(() => {
        window.removeEventListener('keydown', keydown_listener);
		window.removeEventListener('click', click_listener);
	});

	export function selected_value() {
		return selected;
	}

	export function internal_input(): HTMLInputElement {
		return search_input;
	}

	export function reportValidity() {
		return search_input.reportValidity();
	}
</script>

<!-- TODO: BUG: Doesn't activate search if typing fast -->
<div class="relative w-full">
	<input
		type="text"
		bind:this={search_input}
		autocomplete="off"
		id={input_id}
		class="w-full h-full border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent relative z-auto"
		placeholder={input_placeholder}
		on:click={() => {
			// Toggle the dropdown
			if (dropdown_div.classList.contains('hidden')) {
				dropdown_div.classList.remove('hidden');

				// Reset the search results
				search_input.value = '';
				search_results = [];
			} else {
				close();
			}
		}}
		on:input={typing_handler}
	/>

	<div
		bind:this={dropdown_div}
		class="absolute mt-10 top-0 left-0 w-full bg-white dark:bg-custom-dark border dark:border-custom-dark-outline
         border-custom-light-outline shadow-lg dark:shadow-custom-dark-shadow shadow-custom-light-shadow rounded-md
         overflow-hidden overflow-y-auto z-40 hidden"
		style="max-height: {max_dropdown_height};"
	>
		<div class="flex flex-col">
			{#if search_error !== null}
				<div class="flex flex-row items-center p-2 m-3 mt-5">
					<Loader
						blur_background={false}
						icon={'error'}
						icon_size={1}
						ellipsis={false}
						text={'Error searching'}
					/>
				</div>
			{:else if loading}
				<div class="flex flex-row items-center p-2 m-3 mt-5">
					<Loader blur_background={false} icon={'dots'} icon_size={1} text={'Searching'} />
				</div>
			{:else}
				{#each search_results as result, i}
					<button
						class="flex flex-row items-center p-2 dark:hover:bg-custom-darker hover:bg-custom-light"
						on:click={() => select(i)}
						type="button"
					>
						<span class="text-sm">
							{display_map_fn(result)}
						</span>
					</button>
				{:else}
					<div class="flex flex-row items-center p-2">
						<span class="text-sm">No results</span>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>
