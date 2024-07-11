<script lang="ts">
	import '../app.css';

	import { SvelteToast } from '@zerodevx/svelte-toast';
</script>

<SvelteToast
	options={{
		pausable: true
	}}
/>
<slot />
