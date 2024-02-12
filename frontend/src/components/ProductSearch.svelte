<script lang="ts">
	import { api_call, type Product } from '$lib/backend';
	import Loader from './Loader.svelte';

	let current_search = '';
	let results: Product[] = [];

	let search_timeout: null | number = null;

	let results_loading = false;

	export function search(initial: string): string | null {
		document.querySelector('.modal')!.classList.add('is-active');

		let search_bar = document.querySelector('#search-bar')! as HTMLInputElement;

		search_bar.focus();

		// Register an event listener for changes
		search_bar.addEventListener('input', function () {
			if (search_timeout) {
				clearTimeout(search_timeout);
			}

			search_timeout = setTimeout(() => {
				// Clear results
				results = [];

				// Spinner
				results_loading = true;

				// Fetch results
				api_call(`search/product?query=${search_bar.value}&count=10&distance=0.35`, 'GET').then(
					(res) => {
                        res?.json().then((data) => {                           
                            //@ts-ignore
                            results = [...data];
                            results_loading = false;
                        });
					}
				);
			}, 500);
		});

		// Wait for enter

		// Remove event listeners

		// Close

		// Return

		return null;
	}
</script>

<div class="modal">
	<div class="modal-background"></div>
	<div class="modal-content">
		<div class="box">
			<div class="field">
				<p class="control has-icons-left">
					<input
						class="input"
						type="text"
						id="search-bar"
						placeholder="Search"
						bind:value={current_search}
					/>
					<span class="icon is-small is-left">
						<i class="fas fa-search"></i>
					</span>
				</p>
			</div>
			{#if results_loading}
				<Loader />
			{:else}
				<div style="overflow-y: auto;">
					<table class="table is-fullwidth is-hoverable is-striped">
						<tbody>
							{#each results as result (result)}
								<tr>
									<td>{result.name}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	</div>
</div>
