<script lang="ts">
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import { type UserPermissionEnum } from '$bindings/UserPermissionEnum';
	import { api_call } from '$lib/backend';
	import { type ListUserData } from '$bindings/ListUserData';

    import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
    onMount(async () => {
		showNavbar.set(true);
    });

	let users_list: ListUserData[] = [];

	refresh_users();

	function refresh_users() {
		api_call('auth/list_users', 'GET', null).then((res) => {
			if (res?.status == 200) {
				res?.json().then((data) => {
					users_list = [...data];
				});
			}
		});
	}

	let permission_variants: UserPermissionEnum[] = [];

	user_permission_variants().then((res) => {
		if (res) {
			permission_variants = res;
		}
	});

	async function user_permission_variants(): Promise<UserPermissionEnum[] | null> {
		let data = await api_call('auth/permissions', 'GET', null);

		if (data?.status == 200) {
			return data.json();
		} else {
			return null;
		}
	}

	let username = '';
	let password = '';
	let permissions: UserPermissionEnum[] = [];

	function create_user(username: string, password: string, permissions: UserPermissionEnum[]) {
		api_call('auth/create_user', 'POST', {
			username,
			password,
			permissions
		}).then((res) => {
			if (res?.status == 200) {
				refresh_users();
			} else if (res?.status == 403 || res?.status == 401) {
				alert('You are not authorized to add users');
			}
		});
	}

	function delete_user(username: string) {
		api_call('auth/delete_user', 'DELETE', {
			username
		}).then((res) => {
			if (res?.status == 200) {
				refresh_users();
			} else if (res?.status == 403 || res?.status == 401) {
				alert('You are not authorized to delete users');
			}
		});
	}
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<PermissionGuard permissions={['ADMIN']}>
		<div class="w-full flex flex-row h-fit space-x-3">
			<div
				class="h-fit w-full p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<h1 class="title">User Management</h1>

				<!-- Users list -->
				<div style="overflow-y: auto; outline">
					<table class="table is-fullwidth is-hoverable is-striped">
						<thead>
							<tr>
								<th>Username</th>
								<th>Permissions</th>
							</tr>
						</thead>
						<tbody>
							{#each users_list as user}
								<tr>
									<td>{user.username}</td>
									<td>{user.permissions.join(', ')}</td>
									<td>
										<button class="button is-danger" on:click={() => delete_user(user.username)}
											>Delete</button
										>
									</td>
								</tr>
								<!-- There will never be no users, therefore if users_list is empty, it means that the request is still loading -->
							{:else}
								<tr>
									<td colspan="2">
										<div class="skeleton-block"></div>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<div style="height: 2rem;"></div>

				<!-- Add user -->
				<div class="add-user">
					<h2 class="subtitle">Add User</h2>
					<div class="field">
						<label class="label">Username</label>
						<div class="control">
							<input class="input" type="text" bind:value={username} />
						</div>
					</div>
					<div class="field">
						<label class="label">Password</label>
						<div class="control">
							<input class="input" type="password" bind:value={password} />
						</div>
					</div>
					<div class="field">
						<label class="label">Permissions</label>
						<div class="control">
							{#each permission_variants as permission}
								<label class="checkbox">
									<input type="checkbox" bind:group={permissions} value={permission} />
									{permission}
								</label>
							{/each}
						</div>
					</div>
					<div class="field">
						<div class="control">
							<button
								class="button is-primary"
								on:click={() => create_user(username, password, permissions)}
							>
								Add User
							</button>
						</div>
					</div>
				</div>
			</div>
		</div>

		<div slot="denied">
			<h1 class="title">You are not authorized to view this page</h1>
		</div>
	</PermissionGuard>
</div>
