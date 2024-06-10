<!--
 A component which allows the user to search for a value, and select one from a dropdown list. 
 The dropdown list is populated as the user types.
-->
<script lang="ts" generics="ResultType">
	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';

	export let input_id: string;
	export let input_placeholder: string;
	export let input_custom_classes: string = '';

	/// Milliseconds to wait after the user stops typing before sending the search request
	export let typing_search_delay: number = 500;

	export let search_endpoint: string;
	export let search_perms: UserPermissionEnum[] = [];
	export let search_results: ResultType[] = [];
	export let display_map_fn: (val: any) => string;

	let search_input: HTMLInputElement;
	let dropdown_div: HTMLDivElement;

	// Escape handler if dropdown is open
	window.addEventListener('keydown', (e) => {
		if (e.key === 'Escape') {
			search_input.blur();
			dropdown_div.classList.add('hidden');
		}
	});
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
                search_input.blur();
				dropdown_div.classList.add('hidden');
			}
		}}
	/>

	<div
		bind:this={dropdown_div}
		class="absolute mt-10 top-0 left-0 w-full bg-white dark:bg-custom-dark border dark:border-custom-dark-outline
         border-custom-light-outline shadow-lg dark:shadow-custom-dark-shadow shadow-custom-light-shadow rounded-md
         overflow-hidden max-h-96 overflow-y-auto z-40 hidden"
	>
		<div class="flex flex-col">
			{#each [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20] as i}
				<button
					class="flex flex-row items-center p-2 dark:hover:bg-custom-darker hover:bg-custom-light"
				>
					<span class="text-sm">Result {i}</span>
				</button>
			{/each}
			{#if search_results.length < 1}
				<div class="flex flex-row items-center p-2">
					<span class="text-sm">No results</span>
				</div>
			{/if}
		</div>
	</div>
</div>
