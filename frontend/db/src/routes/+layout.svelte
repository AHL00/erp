<script lang="ts">
	import { api_call, get_setting } from '$lib/backend';
	import '../app.css';

	import { SvelteToast } from '@zerodevx/svelte-toast';

	let favicon_url = ``;

	get_setting('logo_low_resolution')
		.then((res) => {
			if (res) {
				// @ts-ignore
				favicon_url = res.ImageBase64URI;
			}
		})
		.catch((err) => {
			console.error(err + ', using default favicon instead.');
		});
</script>

<svelte:head>
	<link rel="icon" href={favicon_url} />
</svelte:head>

<SvelteToast
	options={{
		pausable: true
	}}
/>
<slot />
