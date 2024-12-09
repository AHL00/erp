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
	export let display_extra_map_fn: (val: any) => string | null = () => null;
	export let initial_value: ResultType | null = null;
	export let max_dropdown_height: string = '300px';
	export let form_id: string;
	export let required: boolean = false;
	export let validity_message: string = 'Select a value from the dropdown';
	export let on_change: (val: ResultType) => void = () => {};
	export let on_initial_value: (val: ResultType) => void = () => {};
	export let disabled = false;
	export let classes = '';
	export let presearch_fn: (search: string) => string = (search) => search;

	let search_input: HTMLInputElement;
	let fake_search_input: HTMLInputElement;
	let dropdown_div: HTMLDivElement;
	let dropdown_blur: HTMLDivElement;

	let current_search = '';
	let loading = false;
	let search_error: string | null = null;

	let selected: ResultType | null = null;

	/// If initial set, on_change will not be called. on_initial_value will be called instead.
	function select(idx: number, initial = false) {
		selected = search_results[idx];

		// Set the search input to the selected value
		search_input.value = display_map_fn(selected);
		fake_search_input.value = display_map_fn(selected);

		if (initial) {
			on_initial_value(selected);
		} else {
			on_change(selected);
		}

		close();
	}

	/// Set the selected value
	/// If initial is true, on_initial_value will be called instead of on_change
	/// This is useful to not trigger unwanted behaviour when setting the initial value
	export function set_selected_value(value: ResultType, initial = false) {
		selected = value;
		search_input.value = display_map_fn(selected);
		fake_search_input.value = display_map_fn(selected);

		if (initial) {
			on_initial_value(selected);
		} else {
			on_change(selected);
		}
		close();
	}

	export function remove_selected_value() {
		selected = null;
		search_input.value = '';
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
		dropdown_blur.classList.add('hidden');
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
			search: presearch_fn(search_input.value),
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

	// let click_listener = (e: MouseEvent) => {
	// 	if (dropdown_div !== null && dropdown_div !== undefined) {
	// 		if (!dropdown_div.contains(e.target as Node) && !search_input.contains(e.target as Node)) {
	// 			close();
	// 		}
	// 	}
	// };

	onMount(() => {
		// Hide everything
		dropdown_div.classList.add('hidden');
		dropdown_blur.classList.add('hidden');

		if (required) {
			search_input.required = true;
		}

		search_input.setAttribute('form', form_id);
		// search_input.setCustomValidity(validity_message);

		// Escape handler if dropdown is open
		window.addEventListener('keydown', keydown_listener);

		// Click handler
		// window.addEventListener('click', click_listener);

		if (initial_value !== null) {
			set_selected_value(initial_value, true);
		}
	});

	onDestroy(() => {
		window.removeEventListener('keydown', keydown_listener);
		// window.removeEventListener('click', click_listener);
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
<div class="relative w-full {disabled ? 'cursor-not-allowed opacity-40' : 'cursor-pointer'} {classes}">
	<input
		type="text"
		bind:this={fake_search_input}
		autocomplete="off"
		id={input_id}
		{disabled}
		placeholder={input_placeholder}
		class="w-full h-full border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent relative z-auto pr-10"
		on:focusin={() => {
			dropdown_div.classList.remove('hidden');
			dropdown_blur.classList.remove('hidden');

			// Reset the search results
			search_input.value = '';
			search_results = [];

			// Focus on dropdown search bar
			search_input.focus();
		}}
	/>
	<i
		class="fas fa-search absolute right-2 pb-[1px] top-1/2 transform -translate-y-1/2 opacity-30 pointer-events-none"
	></i>

	<div
		class="z-[41] fixed w-[50vw] h-[50vh] top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 flex flex-col
            bg-white dark:bg-custom-dark border dark:border-custom-dark-outline border-custom-light-outline
                shadow-lg dark:shadow-custom-dark-shadow shadow-custom-light-shadow rounded-md overflow-hidden
                overflow-y-auto p-3 space-y-3"
		bind:this={dropdown_div}
	>
		<div class="flex-shrink-0 h-12 relative overflow-hidden">
			<input
				type="text"
				bind:this={search_input}
				autocomplete="off"
				id={input_id}
				{disabled}
				placeholder={selected ? display_map_fn(selected) : input_placeholder}
				class="h-full w-full border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent relative z-auto pr-10"
				on:focusin={() => {}}
				on:input={typing_handler}
			/>
			<i
				class="fas fa-search absolute right-2 pb-[1px] top-1/2 transform -translate-y-1/2 opacity-30 pointer-events-none"
			></i>
		</div>
		<div class="flex flex-col overflow-auto">
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
						<span class="text-md">
							{display_map_fn(result)}
						</span>
						{#if display_extra_map_fn(result) !== null}
							<div class="h-4 border-r border-gray-400 mx-2"></div>
							<span class="text-md italic">
								{display_extra_map_fn(result)}
							</span>
						{/if}
					</button>
				{:else}
					<div class="flex flex-row items-center p-2">
						<span class="text-sm">No results</span>
					</div>
				{/each}
			{/if}
		</div>
	</div>
	<div
		class="z-40 backdrop-blur-md bg-opacity-35 fixed w-[100vw] h-[100vh] top-0 left-0"
		bind:this={dropdown_blur}
		on:click={close}
	></div>
</div>
