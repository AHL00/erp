<script lang="ts">
	import { login, auth_info_store, LoginStatus } from '$lib/auth';
	import { get } from 'svelte/store';
	import { page } from '$app/stores';
	import { api_call, get_setting } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';

	let username = '';
	let password = '';

	function handleSubmit() {
		// Handle login here
		let login_promise = login(username, password);

		login_promise.then((status) => {
			if (status == LoginStatus.SUCCESS) {
				console.log('Checking auth status');
				if (get(auth_info_store) !== null) {
					// Redirect to app which will check for auth
					// TODO: Redirect to the page the user was trying to access
					let url_params = new URLSearchParams(window.location.search);
					let redirect = url_params.get('redirect');
					// TODO: Just redirect to /app cause redirecting to orders/edit for example breaks loader stuff for some reason. Easier just to
					// redirect to /app and let the app handle the redirect
					console.log('Redirecting to', redirect);

					window.location.href = redirect ? redirect : '/app';
				} else {
					toast.push('Login failed');
				}
			} else if (status == LoginStatus.FAILED_TO_REACH_SERVER) {
				toast.push('Failed to reach server');
			} else if (status == LoginStatus.INCORRECT_CREDENTIALS) {
				toast.push('Username or password is incorrect');
			} else if (status == LoginStatus.SERVER_ERROR) {
				toast.push('Internal server error');
			} else {
				toast.push('Login failed');
			}
		});
	}

	let logo_url = ``;

	api_call('settings/get_one/logo_high_resolution', 'GET', null)
		.then((res) => {
			if (res) {
				if (res.status === 200) {
					res.json().then((data) => {
						let value = data.value['ImageBase64URI'];
						if (value !== null && value !== undefined && value !== '') {
							logo_url = value;
						} else {
							throw new Error('No logo url found in settings');
						}
					});
				} else {
					throw new Error('Error fetching favicon url: ' + res.status);
				}
			} else {
				throw new Error('No response from server when fetching favicon url');
			}
		})
		.catch((err) => {
			console.error(err + ', using default favicon instead.');
		});

	let business_name = ``;

	get_setting('business_name')
		.then((res) => {
			// @ts-ignore
			business_name = res['Text'];
		})
		.catch((err) => {
			console.error(err);
		});
</script>

<div class="h-screen w-screen flex items-center">
	<div
		class="h-auto w-96
        bg-custom-lighter dark:bg-custom-dark
        rounded-lg shadow-lg
        shadow-custom-light-shadowdark:shadow-custom-dark-shadow
        m-auto px-7 pt-9 pb-7"
	>
		<div class="h-full w-full flex flex-col items-center place-content-between space-y-10">
			<div class="flex flex-col items-center space-y-4">
				<object data={logo_url} type="image/png" class="w-36 h-36 rounded-xl" aria-label="Logo">
				</object>
				<h1 class="text-2xl font-bold text-center">{business_name}</h1>
			</div>
			<!-- <h1 class="text-2xl font-bold text-center">Login</h1> -->
			<form class="w-full flex-col space-y-5" on:submit|preventDefault={handleSubmit}>
				<div class="flex flex-col space-y-3">
					<input
						type="text"
						class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
						placeholder="Username"
						id="username"
						bind:value={username}
						required
					/>
					<input
						type="password"
						class="w-full box-border border dark:border-custom-dark-outline border-custom-light-outline text-sm rounded p-2 bg-transparent"
						placeholder="Password"
						id="password"
						bind:value={password}
						required
					/>
				</div>
				<div class="flex flex-col items-center">
					<button
						type="submit"
						class="w-2/5 font-semibold p-2 rounded-xl
                        text-custom-text-light dark:text-custom-text-dark-darker
                        border-custom-light-outline dark:border-custom-dark-outline
                        shadow-custom-light-shadow dark:shadow-custom-dark-shadow
                        shadow-sm
                        border
                        hover:bg-custom-light dark:hover:bg-custom-darker
                        hover:scale-105 transform transition duration-200 ease-in-out
                        ">Login</button
					>
				</div>
			</form>
		</div>
	</div>
</div>
