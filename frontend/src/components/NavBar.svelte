<script lang="ts">
	import { middle_ellipsis, redirect } from '$lib';
	import { logout, auth_info_store } from '$lib/auth';
	import PermissionGuard from './PermissionGuard.svelte';
</script>

<div
	class="sidebar bg-white dark:bg-custom-bg-dark !fixed shadow-md dark:shadow-custom-bg-dark-shadow shadow-custom-bg-light-shadow"
>
	<button class="sidebar-item" on:click={() => redirect('/app')}>
		<div class="sidebar-icon">
			<i class="fa fa-home"></i>
		</div>
		<span class="sidebar-label">Home</span>
	</button>

	<div class="separator"></div>

	<button class="sidebar-item" on:click={() => redirect('/app/orders')}>
		<div class="sidebar-icon">
			<i class="fa fa-clipboard-list"></i>
		</div>
		<span class="sidebar-label">Orders</span>
	</button>

	<div class="separator"></div>

    <PermissionGuard permissions={['INVENTORY_READ']}>
	<button class="sidebar-item" on:click={() => redirect('/app/inventory')}>
		<div class="sidebar-icon">
			<i class="fa fa-warehouse"></i>
		</div>
		<span class="sidebar-label">Inventory</span>
	</button>
    </PermissionGuard>

    <div class="separator"></div>

    <PermissionGuard permissions={['CUSTOMERS_READ']}>
	<button class="sidebar-item" on:click={() => redirect('/app/customers')}>
		<div class="sidebar-icon">
			<i class="fa fa-users-gear"></i>
		</div>
		<span class="sidebar-label">Customers</span>
	</button>
    </PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['ADMIN']}>
		<button class="sidebar-item" on:click={() => redirect('/app/admin')}>
			<div class="sidebar-icon">
				<i class="fa fa-lock"></i>
			</div>
			<span class="sidebar-label">Admin</span>
		</button>

		<div class="separator"></div>
	</PermissionGuard>

	<div class="separator mt-auto"></div>

	<button class="sidebar-item" on:click={logout}>
		<div class="sidebar-icon">
			<i class="fa fa-user"></i>
		</div>
		<span class="sidebar-label">
			{middle_ellipsis($auth_info_store ? $auth_info_store.username : '?', 10)}
		</span>
	</button>
</div>

<div class="sidebar-spacer"></div>

<style>
	.sidebar {
		width: 60px;
		height: 100vh;
		transition: width 0.2s ease-in-out;
		overflow: hidden;
		align-self: start;
		justify-self: start;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding-top: 10px;
		padding-bottom: 10px;
	}

	.sidebar-spacer {
		width: 60px;
		height: 100vh;
		transition: width 0.2s ease-in-out;
	}

	.sidebar:hover {
		width: 200px;
	}

	.sidebar:hover + .sidebar-spacer {
		width: 200px;
	}

	.sidebar:hover > .sidebar-item > .sidebar-label {
		opacity: initial;
		transition: all 0.35s;
		width: 100%;
	}

	.sidebar > .sidebar-item > .sidebar-label {
		transition: all 0.2s;
		opacity: 0;
		font-size: 18px;
		width: 0;
		align-self: center;
		text-align: center;
	}

	.sidebar:hover > .sidebar-item > .sidebar-icon {
		opacity: 0;
		transition: all 0.2s;
		width: 0;
	}

	.sidebar > .sidebar-item > .sidebar-icon {
		transition: all 0.35s;
		width: 100%;
	}

	.sidebar i {
		font-size: 60% !important;
	}

	.sidebar-item {
		width: 100%;
		height: 60px;
		font-size: 28px;
		cursor: pointer;
		transition: all 0.15s;
		display: flex;
		flex-direction: row;
		align-items: center;
	}

	.sidebar-item-no-switch {
		width: 100%;
		height: 60px;
		font-size: 28px;
		cursor: pointer;
		transition: all 0.15s;
		display: flex;
		flex-direction: row;
		align-items: center;
	}

	.sidebar-item-no-switch > .sidebar-icon {
		width: 100%;
	}

	.separator {
		/* height: 2px;
		margin-top: 9px;
		margin-bottom: 9px; */
	}

	.sidebar-item:hover {
		scale: 1.1;
	}
</style>
