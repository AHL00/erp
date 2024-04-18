<script lang="ts">
	import { auth_info_store, refreshAuthStatus } from '$lib/auth';
	import { onMount } from 'svelte';
	import NavBar from '../../components/NavBar.svelte';
	import Loader from '../../components/Loader.svelte';

	// let authenticated = false
	// while (!authenticated) {
	//     refreshAuthStatus().then(() => {
	//         authenticated = true
	//     }).catch(() => {
	//         authenticated = false

	//     })

	//     // Wait
	// }

	// Try authenticating user, if not working, set error in loader and retry in set amount with count down in loader.text
	let loader: Loader;

	function try_auth() {
		refreshAuthStatus()
			.then(() => {
				loader.change_text('Authenticated. Loading data');
				loader.icon = 'dots';
				loader.enable_ellipsis();
			})
			.catch(() => {
				loader.change_text('Error authenticating user. Retrying in 5 seconds.');
				loader.icon = 'error';
				loader.disable_ellipsis();

				let timeout = 5000;

				let interval = setInterval(() => {
					timeout -= 1000;
					if (timeout <= 0) {
						clearInterval(interval);
                        loader.change_text('Error authenticating user. Retrying');
                        loader.enable_ellipsis();
						try_auth();
					} else {
						loader.change_text(`Error authenticating user. Retrying in ${timeout / 1000} seconds.`);
					}
				}, 1000);
			});
	}

	onMount(() => {
        try_auth();
    });
</script>

{#if $auth_info_store === null}
	<div
		style="display: flex; flex-direction: row; overflow: hidden; height: 100vh; align-items: center; justify-content: center;"
	>
		<Loader blur_background={true} custom_classes="rounded-2xl" bind:this={loader} />
	</div>
{:else}
	<div style="display: flex; flex-direction: row; height: 100vh;">
		<NavBar />

		<div class="flex-grow">
			<slot />
		</div>
	</div>
{/if}
