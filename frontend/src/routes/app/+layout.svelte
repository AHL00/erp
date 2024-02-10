<script lang="ts">
	import { auth_info_store, refreshAuthStatus } from '$lib/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(async () => {
		await refreshAuthStatus();
		if ($auth_info_store === null) {
			// Redirect to login page with a way to return to the current page
			goto('/login?redirect=' + encodeURIComponent(window.location.pathname));
		}
	});
</script>

{#if $auth_info_store === null } 
    <p>Redirecting to login page...</p>
{:else}
    <slot />
{/if}
