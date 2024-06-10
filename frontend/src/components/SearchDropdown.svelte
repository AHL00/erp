<!--
 A component which allows the user to search for a value, and select one from a dropdown list. 
 The dropdown list is populated as the user types.
-->
<script lang="ts" generics="ResultType">
	import { api_call } from '$lib/backend';

	import Loader from './Loader.svelte';

	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';

	export let input_id: string;
	export let input_placeholder: string;
	export let input_custom_classes: string = '';

	/// Milliseconds to wait after the user stops typing before sending the search request
	export let typing_search_delay: number = 500;

	// The endpoint to send the search request to.
	// It must be a POST request accepting the type SearchRequest as the body.
	export let search_endpoint: string;
	export let search_perms: UserPermissionEnum[] = [];
	export let search_results: ResultType[] = [];
	export let display_map_fn: (val: any) => string;

	let search_input: HTMLInputElement;
	let dropdown_div: HTMLDivElement;

	let current_search = '';
	let loading = false;
	let search_error: string | null = null;

	let selected: ResultType | null = null;

	// Escape handler if dropdown is open
	window.addEventListener('keydown', (e) => {
		if (e.key === 'Escape') {
			if (!dropdown_div.classList.contains('hidden')) {
				close();
			}
		}
	});

	function select(idx: number) {
		selected = search_results[idx];

		// Set the search input to the selected value
		search_input.value = display_map_fn(selected);

		close();
	}

	function close() {
		// If closed without selecting, reset the search input
		if (selected === null) {
			search_input.value = '';
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
		loading = true;
		search_error = null;

		// Fetch the search results
		api_call(
			search_endpoint,
			'POST',
			JSON.stringify({
				search: current_search
			})
		)
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
</script>

<div class="relative w-full">
	<input
		type="text"
		bind:this={search_input}
		id={input_id}
		class="w-full h-full border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
		placeholder={input_placeholder}
		on:click={() => {
			// If the dropdown is shown, hide it
			if (dropdown_div.classList.contains('hidden')) {
				dropdown_div.classList.remove('hidden');
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
         overflow-hidden max-h-96 overflow-y-auto z-40 hidden"
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
