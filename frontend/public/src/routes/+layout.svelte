<script lang="ts">
	import { api_call } from '$lib/backend';
	import '../app.css';

	import { SvelteToast } from '@zerodevx/svelte-toast';

	let favicon_url = ``;

	api_call('settings/get_one/logo_low_resolution', 'GET', null)
		.then((res) => {
			if (res) {
				if (res.status === 200) {
					res.json().then((data) => {
						let value = data.value['ImageBase64URI'];
						if (value !== null && value !== undefined && value !== '') {
							favicon_url = value;
						} else {
							throw new Error('No favicon url found in settings');
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
