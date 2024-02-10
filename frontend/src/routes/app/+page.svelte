<script lang="ts">
    import { logout } from '$lib/auth';

</script>

<h1>Welcome to the ERP</h1>

<button on:click={logout}>Logout</button>