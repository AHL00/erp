<script lang="ts">
	import { login, auth_info_store } from '$lib/auth';
	import { get } from 'svelte/store';
	let username = '';
	let password = '';

	function handleLogin() {
		// Handle login here
		let login_promise = login(username, password);

		login_promise.then(() => {
            console.log("Checking auth status");
			if (get(auth_info_store) !== null) {
				// Redirect to app which will check for auth
				// TODO: Redirect to the page the user was trying to access
				window.location.href = '/app';
			} else {
				// TODO: Show error message
				alert('Login failed');
			}
		});
	}
</script>

<div class="login-page">
	<h1>Login</h1>
	<form on:submit|preventDefault={handleLogin}>
		<div class="form-group">
			<label for="username">Username</label>
			<input id="username" bind:value={username} type="text" required />
		</div>
		<div class="form-group">
			<label for="password">Password</label>
			<input id="password" bind:value={password} type="password" required />
		</div>
		<button type="submit" disabled={!username || !password}> Login </button>
	</form>
</div>

<style>
	.login-page {
		width: 300px;
		margin: 0 auto;
		padding: 1em;
	}
	.form-group {
		margin-bottom: 1em;
	}
	label {
		display: block;
		margin-bottom: 0.5em;
	}
	input {
		width: 100%;
		padding: 0.5em;
	}
	button {
		padding: 0.5em 1em;
	}
</style>
