<script lang="ts">
	import { CollapsibleCard } from 'svelte-collapsible';
    import { v4 as uuidv4} from 'uuid';

    const sizes = ["sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl"];

    type HeaderSize = "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl";
    export let header_size: HeaderSize = "md";
    export let header_text = "";

    function getOneSizeAbove(size: HeaderSize) {
        return sizes[sizes.indexOf(size) + 1];
    }

    let id = uuidv4();
</script>

<CollapsibleCard>
	<div
		slot="header"
		class="flex flex-row space-x-3 w-full h-fit collapsible-div-{id}"
		on:click={() => {
			document.querySelectorAll(`.collapsible-div-${id} i`).forEach((i) => {
				i.classList.toggle('collapse-active');
			});
		}}
	>
		<span class="font-bold text-{getOneSizeAbove(header_size)}"
			><i class="fa-solid fa-chevron-right mr-2 collapse-active text-{header_size}"></i> {header_text}</span
		>
	</div>
    <slot name="body" slot="body"/>
</CollapsibleCard>

<style>
	i.collapse-active {
		transform: rotate(90deg);
	}

	i {
		transition: transform 0.3s;
	}
</style>
