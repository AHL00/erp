<script>
	import { onMount } from 'svelte';
    import { auth_info, logout, login } from './auth.js';

	let username = '';
	let password = '';
</script>

{#if $auth_info}
<div>
    <h2>Auth Info:</h2>
    <pre>{JSON.stringify($auth_info, null, 2)}</pre>
    <button on:click={logout}>Logout</button>
</div>
{:else}
<div>
    <input bind:value={username} placeholder="Username" />
    <input bind:value={password} placeholder="Password" type="password" />
    <button on:click={login(username, password)}>Login</button>
</div>
{/if}