<script lang="ts">
	import type { UserPermissionEnum } from '$bindings/UserPermissionEnum';
	import { auth_info_store } from '$lib/auth';

	export let permissions: string[];
	/// This is a variable that will be used to determine if all permissions are required or any
	export let all_or_any: 'all' | 'any' = 'all';

	if (!permissions) {
		throw new Error('No permissions provided to PermissionGuard');
	}

	let allowed = false;

	$: {
		allowed = false;

		if (!($auth_info_store === null || $auth_info_store === undefined)) {
			// If includes ADMIN, then it will be allowed
			if ($auth_info_store.permissions.includes('ADMIN')) {
				allowed = true;
			} else {
				// If it includes all required permissions, then it will be allowed
				if (all_or_any === 'all') {
					allowed = permissions.every((permission) =>
						$auth_info_store.permissions.includes(permission as UserPermissionEnum)
					);
				} else if (all_or_any === 'any') {
					// If it includes any of the required permissions, then it will be allowed
					allowed = permissions.some((permission) =>
						$auth_info_store.permissions.includes(permission as UserPermissionEnum)
					);
				}
			}
		}
	}
</script>

<!-- PermissionGuard.svelte -->
{#if allowed}
	<slot />
{:else}
	<slot name="denied" />
{/if}
