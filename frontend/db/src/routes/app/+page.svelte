<script lang="ts">
	import { get_setting } from "$lib/backend";

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

<svelte:head>
    <title>{business_name}</title>
</svelte:head>
