<script lang="ts">
    type SkeletonTypes = 'table' | 'text' | 'image';

    export let type: SkeletonTypes = 'text';
</script>

<div class="h-full w-full grow p-4">
    {#if type == 'table'}
        <div class="animate-pulse flex space-x-4">
            <div class="flex-1 space-y-4 py-1">
                <div class="h-4 bg-gray-300 rounded w-3/4"></div>
                <div class="h-4 bg-gray-300 rounded w-5/6"></div>
                <div class="h-4 bg-gray-300 rounded w-4/6"></div>
                <div class="h-4 bg-gray-300 rounded w-3/4"></div>
                <div class="h-4 bg-gray-300 rounded w-5/6"></div>
            </div>
        </div>
    {:else if type == 'text'}
        <div class="animate-pulse h-4 bg-gray-300 rounded w-3/4"></div>
    {:else if type == 'image'}
        <div class="animate-pulse h-64 w-64 bg-gray-300 rounded"></div>
    {/if}
</div>