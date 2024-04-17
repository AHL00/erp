<script lang="ts">
    import { onMount } from 'svelte';
    import { writable } from 'svelte/store';

    let text = 'Loading...';

    onMount(() => {
        const interval = setInterval(() => {
            text = text === 'Loading...' ? 'Loading' : text + '.';
        }, 500);

        return () => clearInterval(interval);
    });
</script>

<div
	class="flex-grow flex flex-col place-self-center space-y-3 w-full h-full z-50 justify-center items-center bg-custom-bg-lighter dark:bg-custom-bg-dark"
>
	<div class="flex place-items-center space-x-2 opacity-95">
		<div
			class="h-4 w-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce [animation-delay:-0.3s]"
		></div>
		<div
			class="h-4 w-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce [animation-delay:-0.15s]"
		></div>
		<div
			class="h-4 w-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce"
		></div>
	</div>
	<span class="text-custom-text-light dark:text-custom-text-dark text-xl opacity-95">{text}</span>
</div>

<style>
	.loader-wrapper {
		height: 100%;
		width: 100%;
		background: #fff;
		transition: opacity 0.3s;
		display: flex;
		justify-content: center;
		align-items: center;
		border-radius: 6px;
	}

	.loader-wrapper > .loader {
		height: 80px;
		width: 80px;
	}

	.is-loading {
		position: relative;
	}
</style>
