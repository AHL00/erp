<script lang="ts">
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import { type Setting } from '$bindings/Setting';
	import { api_call } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';

	let settings: Setting[] = [];
	let loading = true;

	async function fetchSettings() {
		try {
			const response = await api_call('/settings/get_all', 'GET', null);

			if (!response) {
				throw new Error('No response');
			}

			if (response.status === 200) {
				settings = await response.json();
				loading = false;
			} else {
				throw new Error(`Response status: ${response.status}`);
			}
		} catch (e) {
            console.error('Failed to fetch settings:', e);
			toast.push('Failed to fetch settings');
		}
	}

	fetchSettings();
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<PermissionGuard permissions={['SETTINGS']}>
		<div class="w-full flex flex-row h-full space-x-3">
			<div
				class="h-full w-full p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<span class="text-3xl">Settings</span>
				{#each settings as setting}
					<div class="flex flex-row justify-between items-center p-2 border-b border-gray-300">
						<span>{setting.key}</span>
						<span>{setting.value}</span>
					</div>
				{/each}
			</div>
		</div>

		<div slot="denied">
			<h1 class="title">You are not authorized to view this page</h1>
		</div>
	</PermissionGuard>
</div>
