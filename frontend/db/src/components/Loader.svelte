<svelte:options accessors />

<script lang="ts">
	import { onMount } from 'svelte';

	export let blur_background = false;
	export let text = 'Loading';
	export let custom_classes = '';
	export let ellipsis = true;

	type Icon = 'dots' | 'spinner' | 'error';
	export let icon: Icon = 'dots';
    export let icon_size: number = 1;

	let displayed_text = `${text}`;

	export function change_text(new_text: string) {
		text = new_text;
		displayed_text = text;
	}

	let ellipsis_interval: any;

	export function disable_ellipsis() {
		if (ellipsis_interval != null) {
			clearInterval(ellipsis_interval);
			ellipsis_interval = null;
		}
	}

	export function enable_ellipsis() {
		if (ellipsis_interval == null) {
			ellipsis_interval = setInterval(() => {
				displayed_text = displayed_text === `${text}...` ? text : displayed_text + '.';
			}, 500);
		}
	}

    let icon_container: HTMLElement;

	onMount(() => {
        icon_container.style.scale = `${icon_size}`;
		
        if (ellipsis) {
			enable_ellipsis();
		}
		
        return () => {
			if (ellipsis_interval != null) {
				clearInterval(ellipsis_interval);
			}
		};
	});
</script>

<div
	class="flex-grow flex flex-col place-self-center space-y-3 w-full h-full z-30 justify-center items-center bg-transparent {custom_classes}"
	class:backdrop-blur-md={blur_background}
	class:!bg-opacity-35={blur_background}
>
	<div class="flex place-items-center space-x-2 opacity-95" bind:this={icon_container}>
		{#if icon === 'dots'}
			<div
				class="h-4 w-4 mt-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce [animation-delay:-0.3s]"
			></div>
			<div
				class="h-4 w-4 mt-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce [animation-delay:-0.15s]"
			></div>
			<div
				class="h-4 w-4 mt-4 bg-custom-text-light dark:bg-custom-text-dark rounded-full animate-bounce"
			></div>
		{/if}
		{#if icon === 'spinner'}
			<svg
				class="animate-spin h-6 w-6 text-custom-text-light dark:text-custom-text-dark"
				fill="none"
				viewBox="0 0 24 24"
			>
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
				></circle>
				<path
					class="opacity-75"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A8.001 8.001 0 0112 4V0C6.486 0 2 4.486 2 10h4zm7.292 2A8.01 8.01 0 0120 12h4c0-4.418-3.582-8-8-8v4z"
					fill="currentColor"
				></path>
			</svg>
		{/if}
		{#if icon === 'error'}
			<div class="text-6xl animate-pulse">
				<i class="fa fa-triangle-exclamation"></i>
			</div>
		{/if}
	</div>
	<span class="text-custom-text-light dark:text-custom-text-dark text-xl opacity-95"
		>{displayed_text}</span
	>
</div>
