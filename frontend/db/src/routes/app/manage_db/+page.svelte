<script lang="ts">
	import { toast } from '@zerodevx/svelte-toast';
	import CollapsibleWithArrowHeader from '../../../components/CollapsibleWithArrowHeader.svelte';
	import PermissionGuard from '../../../components/PermissionGuard.svelte';
	import { api_call } from '$lib/backend';

    import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
    onMount(async () => {
		showNavbar.set(true);
    });

	let creating_backup = false;
	async function createBackup() {
		creating_backup = true;
		try {
			const response = await api_call('db/backup', 'GET', null);
            if (!response) {
                throw new Error('No response from server');
            }
            if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			const blob = await response.blob();
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.style.display = 'none';
			a.href = url;
			a.download = `backup-${new Date().toISOString()}.sql`;
			document.body.appendChild(a);
            toast.push('Backup created and downloading');
			a.click();
			window.URL.revokeObjectURL(url);
		} catch (error) {
			toast.push('Failed to create backup');
			console.error('There was a problem with the fetch operation:', error);
		}
		creating_backup = false;
	}

    let restoring_backup = false;
    async function restoreBackup() {
        restoring_backup = true;
        
        // Open file dialog
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.sql';
        input.onchange = async (e) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (!file) {
                toast.push('No file selected');
                restoring_backup = false;
                return;
            }

            let fileData = await file.text();

            let confirmed = confirm('Are you sure you want to restore this backup? This will overwrite the current database including users and passwords');

            if (!confirmed) {
                toast.push('Restore cancelled');
                restoring_backup = false;
                return;
            }

            try {
                const response = await api_call('db/restore', 'POST', fileData);
                if (!response) {
                    throw new Error('No response from server');
                }
                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }
                toast.push('Backup restored');
            } catch (error) {
                toast.push('Failed to restore backup');
                console.error('There was a problem with the fetch operation:', error);
            }
        };

        input.click();

        restoring_backup = false;
    }
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3 space-y-3">
	<PermissionGuard permissions={['MANAGE_DB']}>
		<div class="w-full flex flex-row h-fit space-x-3">
			<div
				class="h-fit w-full p-2 pl-4 rounded-lg shadow-md bg-custom-lighter dark:bg-custom-dark flex flex-col"
			>
				<CollapsibleWithArrowHeader header_text="Backups" header_size="xl">
					<div slot="body" class="flex flex-col space-y-3 pb-3">
						<button class="btn btn-primary" on:click={createBackup} disabled={creating_backup}>
                            {creating_backup ? 'Creating backup...' : 'Create backup'}
                        </button>
                        <button class="btn btn-secondary" on:click={restoreBackup} disabled={restoring_backup}>Restore backup</button>
					</div>
				</CollapsibleWithArrowHeader>
			</div>
		</div>

		<div slot="denied">
			<h1 class="title">You are not authorized to view this page</h1>
		</div>
	</PermissionGuard>
</div>
