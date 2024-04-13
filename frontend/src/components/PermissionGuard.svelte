
<script lang="ts">
	import type { UserPermission } from "$bindings/UserPermission";
	import { auth_info_store } from "$lib/auth";

    export let permissions: string[];

    if (!permissions) {
        throw new Error('No permissions provided to PermissionGuard');
    }

    let allowed = false;

    $: {
        allowed = false;

        // If includes ADMIN, then it will be allowed
        if ($auth_info_store?.permissions.includes('ADMIN')) {
            allowed = true;
        } else {
            // If it includes all required permissions, then it will be allowed
            allowed = permissions.every(permission => $auth_info_store?.permissions.includes(permission as UserPermission));
        }
    }
</script>

<!-- PermissionGuard.svelte -->
{#if allowed}
    <slot></slot>
{:else}
    <slot name="denied"></slot>
{/if}