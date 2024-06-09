<!-- Right aligned sidebar which is fixed positioned -->
<script lang="ts">
	let sidebar: Element;
	let content_div: Element;

	let currently_open = false;
	let last_opened_at = 0;

	export function toggle_sidebar() {
		if (currently_open) close_sidebar();
		else open_sidebar();

		currently_open = !currently_open;

		// Force repaint/reflow to trigger transition
		void sidebar!.clientHeight;
	}

	export function close_sidebar() {
		sidebar!.classList.remove('open-sidebar');
		content_div!.classList.remove('open-content');
		content_div!.classList.remove('blur-md');
		currently_open = false;

		// Force repaint/reflow to trigger transition
		void sidebar!.clientHeight;
	}

	export function open_sidebar() {
		sidebar!.classList.add('open-sidebar');
		content_div!.classList.add('open-content');
		content_div!.classList.add('blur-md');
		currently_open = true;
		last_opened_at = Date.now();

		// Force repaint/reflow to trigger transition
		void sidebar!.clientHeight;
	}

	export let width: string;
	export let close_on_click_outside: boolean = true;
	/// Callback to be called when the sidebar is closed, this is useful when the sidebar is closed by clicking outside
	export let close_callback: () => void = () => {};
</script>

<!-- min-h-0 prevents the flex-grow from overflowing the parent container -->
<div class="flex flex-row flex-grow min-h-0">
	<div
		bind:this={content_div}
		style="--width: {width}"
		class="flex-grow content-div"
		on:click={() => {
			if (close_on_click_outside && currently_open) {
				// If the click was within 200ms of the sidebar being opened, don't close it
				if (Date.now() - last_opened_at < 200) return;
				close_sidebar();
				close_callback();
			}
		}}
	>
		<slot name="content" />
	</div>

	<div
		bind:this={sidebar}
		style="--width: {width}"
		class="sidebar !fixed flex flex-col justify-start bg-white dark:bg-custom-bg-dark shadow-md dark:shadow-custom-bg-dark-shadow shadow-custom-bg-light-shadow"
	>
		<slot name="sidebar" />
	</div>
</div>

{#if false}
	<!-- Gets rid of unused css getting optimised out -->
	<div class="open-content open-sidebar"></div>
{/if}

<style>
	.content-div {
		padding-right: 0;
		transition: 0.2s cubic-bezier(0.36, -0.01, 0, 0.77);
	}

	.sidebar {
		width: var(--width);
		height: 100vh;
		top: 0;
		right: 0;
		margin-right: calc(-1 * var(--width));
		transition: 0.2s cubic-bezier(0.36, -0.01, 0, 0.77);
	}

	.open-content {
		padding-right: var(--width);
	}

	.open-sidebar {
		margin-right: 0;
	}
</style>
