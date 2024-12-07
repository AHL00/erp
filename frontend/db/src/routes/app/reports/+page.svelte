<script lang="ts">
    import Expense from "./Expense.svelte";
import Main from "./Main.svelte"
	import Orders from "./Orders.svelte";

    import { showNavbar } from '../../../stores/navbarStore';
	import { onMount } from 'svelte';
    onMount(async () => {
		showNavbar.set(true);
    });

    const tabs = ["expenses", "orders"];

    let current_tab = "orders";

    function capitaliseFirstLetter(string: string) {
        return string.charAt(0).toUpperCase() + string.slice(1);
    }
</script>

<svelte:head>
    <title>Reports</title>
</svelte:head>

<div class="flex flex-col w-full">
    <div class="flex flex-row w-full px-1 py-2 dark:bg-custom-dark bg-custom-lighter 
    dark:shadow-custom-dark-shadow shadow-custom-light-shadow
    shadow-md">
        {#each tabs as tab}
            <button
                class="
                w-36 mx-1 p-2 text-center rounded-lg 
                dark:bg-custom-darker bg-custom-light
                dark:shadow-custom-dark-shadow shadow-custom-light-shadow
                dark:outline-custom-dark-outline outline-custom-light-outline
                {current_tab !== tab ? 
                'shadow-inner' : 
                'outline outline-2 shadow-md'}
                "
                class:active={current_tab === tab}
                on:click={() => current_tab = tab}
            >
                <span class={current_tab !== tab ? 
                    'opacity-40' : 
                    'opacity-100'}>
                    {capitaliseFirstLetter(tab)}
                </span>
            </button>
        {/each}
    </div>
    {#if current_tab === 'finance'}
        <Main/>
    {/if}
    {#if current_tab === 'expenses'}
        <Expense/>
    {/if}
    {#if current_tab === 'orders'}
        <Orders/>
    {/if}
</div>

