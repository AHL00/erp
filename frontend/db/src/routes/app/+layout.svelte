<script lang="ts">
	import { auth_info_store, auth_status, auth_status_store, refreshAuthStatus } from '$lib/auth';
	import { onMount } from 'svelte';
	import NavBar from '../../components/NavBar.svelte';
	import Loader from '../../components/Loader.svelte';
	import FullscreenLoader from '../../components/FullscreenLoader.svelte';
	import { showNavbar } from '../../stores/navbarStore';

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
	let loader: FullscreenLoader;

	// export function try_auth() {
	// 	refreshAuthStatus()
	// 		.then((status: boolean) => {
	// 			if (status === true) {
	// 				loader.change_text('Authenticated. Loading data');
	// 				loader.icon = 'dots';
	// 				loader.enable_ellipsis();
	// 			} else {
	// 				// Should have redirected to login page
	// 				console.error(
	// 					'User not authenticated and auth_info_store is null, should have redirected to login page'
	// 				);
	// 			}
	// 		})
	// 		.catch(() => {
	// 			loader.change_text('Error authenticating user. Retrying in 5 seconds.');
	// 			loader.icon = 'error';
	// 			loader.disable_ellipsis();

	// 			let timeout = 5000;

	// 			let interval = setInterval(() => {
	// 				timeout -= 1000;
	// 				if (timeout <= 0) {
	// 					clearInterval(interval);
	// 					loader.change_text('Error authenticating user. Retrying');
	// 					loader.enable_ellipsis();
	// 					try_auth();
	// 				} else {
	// 					loader.change_text(`Error authenticating user. Retrying in ${timeout / 1000} seconds.`);
	// 				}
	// 			}, 1000);
	// 		});
	// }

	onMount(() => {
        // Enable by default
        showNavbar.set(true);

		refreshAuthStatus();

		auth_status_store.subscribe((status) => {
			if (status == auth_status.LOADING) {
				loader.show();
				loader.set_text('Authenticating user');
				loader.icon = 'dots';
				loader.enable_ellipsis();
			} else if (status == auth_status.FETCH_ERROR) {
				loader.show();
				loader.set_text('Error authenticating user. Retrying in 5 seconds.');
				loader.icon = 'error';
				loader.disable_ellipsis();

				let timeout = 5000;

				let interval = setInterval(() => {
					timeout -= 1000;
					if (timeout <= 0) {
						clearInterval(interval);
						loader.set_text('Error authenticating user. Retrying');
						loader.enable_ellipsis();
						refreshAuthStatus();
					} else {
						loader.set_text(`Error authenticating user. Retrying in ${timeout / 1000} seconds.`);
					}
				}, 1000);
			} else if (status == auth_status.NOT_AUTHENTICATED) {
				loader.show();
				loader.set_text('User not authenticated. Redirecting to login page');
				loader.icon = 'error';
				loader.enable_ellipsis();
			} else if (status == auth_status.AUTHENTICATED) {
				loader.hide();
			}
		});
	});
</script>

<FullscreenLoader blur_background={true} custom_classes="rounded-2xl" bind:this={loader} />
{#if $auth_info_store !== null && $auth_status_store === auth_status.AUTHENTICATED}
	<div style="display: flex; flex-direction: row; height: 100vh;">
        {#if $showNavbar}
            <NavBar />
        {/if}

		<div class="flex-grow">
			<slot />
		</div>
	</div>
{/if}
