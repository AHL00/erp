<script lang="ts">
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";
    import '$lib/../app.scss';
	import Layout from "./app/+layout.svelte";

    onMount(() => {
        // Redirect to app which will check for auth
        goto("/app");
    });

</script>