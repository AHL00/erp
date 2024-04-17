<script>
	import { auth_info_store, refreshAuthStatus } from '$lib/auth';
	import { onMount } from 'svelte';
	import NavBar from '../../components/NavBar.svelte';
	import Loader from '../../components/Loader.svelte';

	onMount(refreshAuthStatus);
</script>

{#if $auth_info_store === null}
	<!-- TODO Add a loading spinner -->
	<div
		style="display: flex; flex-direction: row; overflow: hidden; height: 100vh; align-items: center; justify-content: center;"
	>
		<Loader />
	</div>
{:else}
	<div style="display: flex; flex-direction: row; height: 100vh;">
		<NavBar />

		<!-- Standard padding on every page -->
		<div class="flex-grow">
			<slot />
		</div>
	</div>
{/if}
