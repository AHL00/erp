<script lang="ts">
	import { middle_ellipsis, redirect } from '$lib';
	import { logout, auth_info_store } from '$lib/auth';
	import { get_setting } from '$lib/backend';
	import PermissionGuard from './PermissionGuard.svelte';

	let logo: string | null = null;

	get_setting('logo_low_resolution')
		.then((res) => {
			// @ts-ignore
			logo = res.ImageBase64URI;
		})
		.catch((err) => {
			console.error(err);
		});
</script>

<div
	class="sidebar bg-white dark:bg-custom-dark !fixed shadow-md dark:shadow-custom-dark-shadow shadow-custom-light-shadow"
>
	<button class="sidebar-item" on:click={() => redirect('/app')}>
		<div class="sidebar-icon">
			{#if logo}
				<img src={logo} alt="Logo" class="sidebar-logo h-9 w-9 m-auto" />
			{:else}
				<i class="fa fa-home"></i>
			{/if}
		</div>
		<span class="sidebar-label">Home</span>
	</button>

	<div class="separator"></div>

	<PermissionGuard permissions={['ORDER_READ', 'ORDER_WRITE']} all_or_any="any">
		<button class="sidebar-item" on:click={() => redirect('/app/orders')}>
			<div class="sidebar-icon">
				<i class="fa fa-clipboard-list"></i>
			</div>
			<span class="sidebar-label">Orders</span>
		</button>
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['PURCHASE_READ', 'PURCHASE_WRITE']} all_or_any="any">
		<button class="sidebar-item" on:click={() => redirect('/app/purchases')}>
			<div class="sidebar-icon">
				<i class="fa fa-cart-shopping"></i>
			</div>
			<span class="sidebar-label">Purchases</span>
		</button>
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['EXPENSES_READ', 'EXPENSES_WRITE']} all_or_any="any">
		<button class="sidebar-item" on:click={() => redirect('/app/expenses')}>
			<div class="sidebar-icon">
				<i class="fa fa-money-check-dollar"></i>
			</div>
			<span class="sidebar-label">Expenses</span>
		</button>
	</PermissionGuard>

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
				<i class="fa fa-people-carry-box"></i>
			</div>
			<span class="sidebar-label">Customers</span>
		</button>
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['SUPPLIERS_READ']}>
		<button class="sidebar-item" on:click={() => redirect('/app/suppliers')}>
			<div class="sidebar-icon">
				<i class="fa fa-truck"></i>
			</div>
			<span class="sidebar-label">Suppliers</span>
		</button>
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['REPORTS']}>
		<button class="sidebar-item" on:click={() => redirect('/app/reports')}>
			<div class="sidebar-icon">
				<i class="fa fa-file-contract"></i>
			</div>
			<span class="sidebar-label">Reports</span>
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
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['MANAGE_DB']}>
		<button class="sidebar-item" on:click={() => redirect('/app/manage_db')}>
			<div class="sidebar-icon">
				<i class="fa fa-database"></i>
			</div>
			<span class="sidebar-label">Database</span>
		</button>
	</PermissionGuard>

	<div class="separator"></div>

	<PermissionGuard permissions={['SETTINGS']}>
		<button class="sidebar-item" on:click={() => redirect('/app/settings')}>
			<div class="sidebar-icon">
				<i class="fa fa-sliders"></i>
			</div>
			<span class="sidebar-label">Settings</span>
		</button>
	</PermissionGuard>

	<div class="separator mt-auto"></div>

	<button class="sidebar-item" on:click={logout}>
		<div class="sidebar-icon">
			<i class="fa fa-user"></i>
		</div>
		<span class="sidebar-label flex flex-row justify-center">
			{#if $auth_info_store === null}
				?
			{:else}
				{middle_ellipsis($auth_info_store.username, 10)}
			{/if}
		</span>
	</button>

	<div id="version-text" class="text-center text-xs text-gray-500 mt-2">
		<span>Version __COMMIT_ID__</span>
	</div>
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

	.sidebar:hover > #version-text {
		height: fit-content;
		transition:
			height 2s ease-in,
			opacity 0.5s ease-in;
		opacity: 100%;
	}

	.sidebar > #version-text {
		height: 0px;
		opacity: 0%;
		transition:
			height 0.2s,
			opacity 0.1s;
	}

	#version-text {
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
		font-size: 60%;
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
