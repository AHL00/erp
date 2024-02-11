<script>
	import { auth_info_store, refreshAuthStatus } from '$lib/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import NavBar from '../../components/NavBar.svelte';
	import Loader from '../../components/Loader.svelte';

    onMount(async () => {
        console.log('Checking auth status');
		await refreshAuthStatus();
		if ($auth_info_store === null) {
			// Redirect to login page with a way to return to the current page
			goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
		}
	});
</script>

{#if $auth_info_store === null}
    <!-- TODO Add a loading spinner -->
    <div style="display: flex; flex-direction: row; overflow: hidden; height: 100vh; align-items: center; justify-content: center;">
        <Loader />
    </div>
{:else}
	<div style="display: flex; flex-direction: row; overflow: hidden; height: 100vh;">
		<NavBar />
		<div style="flex: 1; display: flex; flex-direction: column; padding: 20px;">
			<slot />
		</div>
	</div>
{/if}
