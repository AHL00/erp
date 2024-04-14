<!-- Right aligned sidebar which is fixed positioned -->
<script lang="ts">
	let sidebar: Element;
	let sidebar_spacer: Element;

	export function toggleSidebar() {
		sidebar!.classList.toggle('open');
		sidebar_spacer!.classList.toggle('open');

		// Force repaint/reflow to trigger transition
		void sidebar!.clientHeight;
	}

	export let width: string;
</script>

<div style="display: flex; flex-direction: row; height: 100vh;">
	<div class="flex-grow">
		<slot name="content" />
	</div>

	<div
		bind:this={sidebar_spacer}
		style="--width: {width}"
		class="open sidebar !fixed flex flex-col justify-start bg-white dark:bg-custom-bg-dark shadow-md dark:shadow-custom-bg-dark-shadow shadow-custom-bg-light-shadow"
	>
		<slot name="sidebar" />
	</div>
	<div bind:this={sidebar} style="--width: {width}" class="open sidebar-spacer"></div>
</div>

<style>
	.sidebar {
		width: 0px;
		height: 100vh;
		transition: width 0.2s ease-in-out;
		top: 0;
		right: 0;
	}

	.sidebar-spacer {
		width: 0px;
		height: 100vh;
		transition: width 0.2s ease-in-out;
		top: 0;
		right: 0;
	}

	.open {
		width: var(--width);
	}
</style>
