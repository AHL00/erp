<script lang="ts">
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import { type Setting } from '$bindings/Setting';
	import { api_call, SETTINGS_CACHE_PREFIX } from '$lib/backend';
	import { toast } from '@zerodevx/svelte-toast';
	import type { SettingValue } from '$bindings/SettingValue';

	let settings: Setting[] = [];
	let settings_edit: Setting[] = [];
	let loading = true;

	import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
	onMount(async () => {
		showNavbar.set(true);
	});

	async function fetchSettings() {
		try {
			const response = await api_call('/settings/get_all', 'GET', null);

			if (!response) {
				throw new Error('No response');
			}

			if (response.status === 200) {
				settings = await response.json();
				settings_edit = JSON.parse(JSON.stringify(settings));
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

	function compareSettingValues(a: SettingValue, b: SettingValue): boolean {
		if (typeof a !== typeof b) {
			return false;
		}

		if (Object.keys(a).length !== Object.keys(b).length) {
			return false;
		}

		if (Object.keys(a)[0] !== Object.keys(b)[0]) {
			return false;
		}

		let key = Object.keys(a)[0];
		// @ts-ignore
		let a_val = a[key];
		// @ts-ignore
		let b_val = b[key];

		// Special cases for TextVec and ImageBase64URI
		if (key === 'TextVec' || key === 'ImageBase64URI') {
			// This should be efficient as in basically every case
			// different images will have different lengths,
			// meaning we can skip the actual comparison
			if (a_val.length !== b_val.length) {
				return false;
			}

			if (a_val.length == 0 && b_val.length == 0) {
				return true;
			}

			for (let i = 0; i < a_val.length; i++) {
				if (a_val[i] !== b_val[i]) {
					return false;
				}
			}

			return true;
		}

		return a_val === b_val;
	}

	let saving_indexes: Set<number> = new Set();

	function save_setting(idx: number) {
		const setting = settings_edit[idx];
		saving_indexes.add(idx);
		saving_indexes = saving_indexes;

		api_call(`/settings/set/`, 'POST', setting)
			.then((res) => {
				if (res) {
					if (res.status === 200) {
						toast.push('Setting saved');
						settings[idx] = JSON.parse(JSON.stringify(setting));

						// Update cached settings
						sessionStorage.setItem(
							`${SETTINGS_CACHE_PREFIX}${setting.key}`,
							JSON.stringify(settings[idx].value)
						);
					} else {
						throw new Error(`Response status: ${res.status}`);
					}
				} else {
					throw new Error('No response');
				}

				saving_indexes.delete(idx);
				saving_indexes = saving_indexes;
			})
			.catch((err) => {
				console.error('Failed to save setting:', err);
				toast.push('Failed to save setting');

				saving_indexes.delete(idx);
				saving_indexes = saving_indexes;
			});
	}
</script>

<svelte:head>
	<title>Settings</title>
</svelte:head>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<PermissionGuard permissions={['SETTINGS']}>
		<div class="w-full flex flex-row h-full space-x-3">
			<div
				class="h-full w-full p-1 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<span class="text-3xl m-4">Settings</span>
				{#each settings_edit as setting, i}
					<div
						class="flex flex-row justify-between items-center p-2 border-b border-custom-light-outline dark:border-custom-dark-outline"
					>
						<div class="flex flex-col">
							<span class="text-lg text-custom-text-light dark:text-custom-text-dark"
								>{setting.long_name}</span
							>
							{#if setting.description}
								<span class="font-thin text-sm text-custom-text-light dark:text-custom-text-dark"
									>{setting.description}</span
								>
							{/if}
						</div>
						<div class="flex flex-row space-x-2">
							{#if 'Boolean' in setting.value}
								<input type="checkbox" bind:checked={setting.value['Boolean']} />
							{:else if 'Text' in setting.value}
								<input class="p-2 rounded-lg" type="text" bind:value={setting.value['Text']} />
							{:else if 'Int' in setting.value}
								<input
									class="p-2 rounded-lg"
									type="text"
									value={setting.value['Int']}
									on:input={(e) => {
										// @ts-ignore
										if (e.target.value.length == 0) {
											console.log('a');
											// @ts-ignore
											setting.value['Int'] = 0;
											// @ts-ignore
											e.target.value = 0;
											return;
										}

										// @ts-ignore
										if (e.target.value == '0-') {
											console.log('b');
											// @ts-ignore
											e.target.value = '-';
											// @ts-ignore
											setting.value['Int'] = 0;
											return;
										}

										const pattern = /^-?[0-9]*$/;
										//@ts-ignore
										if (!pattern.test(e.target.value)) {
											// Remove last character
											// @ts-ignore
											e.target.value = e.target.value.slice(0, -1);
										} else {
											// @ts-ignore
											let int = parseInt(e.target.value);

											if (isNaN(int)) {
												// @ts-ignore
												e.target.value = '';
												int = 0;
											}

											// @ts-ignore
											setting.value['Int'] = int;
										}
									}}
								/>
							{:else if 'Float' in setting.value}
								<input
									class="p-2 rounded-lg"
									type="number"
									step="0.01"
									bind:value={setting.value['Float']}
								/>
							{:else if 'UnsignedInt' in setting.value}
								<input
									class="p-2 rounded-lg"
									type="text"
									value={setting.value['UnsignedInt']}
									on:input={(e) => {
										// @ts-ignore
										if (e.target.value.length == 0) {
											// @ts-ignore
											setting.value['UnsignedInt'] = 0;
											// @ts-ignore
											e.target.value = 0;
											return;
										}

										const pattern = /^[0-9]*$/;
										//@ts-ignore
										if (!pattern.test(e.target.value)) {
											// Remove last character
											// @ts-ignore
											e.target.value = e.target.value.slice(0, -1);
										} else {
											// @ts-ignore
											let int = parseInt(e.target.value);

											if (int < 0) {
												// @ts-ignore
												e.target.value = 0;
												int = 0;
											} else if (isNaN(int)) {
												// @ts-ignore
												e.target.value = '';
												int = 0;
											}

											// @ts-ignore
											setting.value['UnsignedInt'] = int;
										}
									}}
								/>
							{:else if 'Decimal' in setting.value}
								<input
									class="p-2 rounded-lg"
									type="number"
									step="0.01"
									value={setting.value['Decimal']}
									on:input={(e) => {
										// @ts-ignore
										setting.value['Decimal'] = e.target.value;
									}}
								/>
							{:else if 'ImageBase64URI' in setting.value}
								<div
									class="flex flex-row space-x-2
                                    bg-custom-light dark:bg-custom-darker p-2 rounded-lg
                                "
								>
									<img
										class="w-20 h-20 rounded-lg"
										src={setting.value['ImageBase64URI']}
										alt="preview"
									/>
									<input
										type="file"
										class="w-28 p-2"
										accept="image/*"
										on:change={(e) => {
											// @ts-ignore
											const file = e.target.files[0];
											const reader = new FileReader();
											let size_limit_kb = 1024;
											if (file.size > size_limit_kb * 1024) {
												toast.push(`Image size too large, max size: ${size_limit_kb}KB`);
												return;
											}
											reader.onload = (e) => {
												// @ts-ignore
												setting.value['ImageBase64URI'] = e.target.result;
												setting.value = setting.value;
											};
											reader.readAsDataURL(file);
										}}
									/>
								</div>
							{:else if 'TextVec' in setting.value}
								<div class="flex flex-col space-y-2">
									{#each setting.value['TextVec'] as text, j}
										<div>
											<input
												class="p-2 rounded-lg"
												type="text"
												bind:value={setting.value['TextVec'][j]}
											/>
											<button
												class="font-bold"
												on:click={() => {
													// @ts-ignore
													setting.value['TextVec'].splice(j, 1);
													setting.value = setting.value;
												}}
											>
												<i class="fas fa-trash ml-2 opacity-80"></i>
											</button>
										</div>
									{/each}
								</div>
								<button
									class="font-bold"
									on:click={() => {
										// @ts-ignore
										setting.value['TextVec'].push('');
										setting.value = setting.value;
									}}
								>
									<i class="fas fa-plus ml-2 opacity-80"></i>
								</button>
							{:else}
								<span class="text-red-500">Error: Unknown setting type</span>
							{/if}
							{#if !compareSettingValues(settings[i].value, setting.value)}
								<button
									class="font-bold"
									on:click={() => {
										save_setting(i);
									}}
								>
									{#if saving_indexes.has(i)}
										<i class="fas fa-spinner animate-spin ml-2 opacity-80"></i>
									{:else}
										<i class="fas fa-save ml-2 opacity-80"></i>
									{/if}
								</button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>

		<div slot="denied">
			<h1 class="title">You are not authorized to view this page</h1>
		</div>
	</PermissionGuard>
</div>
