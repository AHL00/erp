<script lang="ts">
	import { onMount } from 'svelte';
	import ProductSearch from '$lib/../components/ProductSearch.svelte';
    import PermissionGuard from '$lib/../components/PermissionGuard.svelte';

	let items = [{ name: '', quantity: 0, price: 0 }];

	let is_mobile = false;

	function checkMobile() {
		is_mobile = window.innerWidth <= 768;
	}

	onMount(() => {
		checkMobile();
		window.addEventListener('resize', checkMobile);
	});

	function addItem() {
		items = [...items, { name: '', quantity: 0, price: 0 }];
	}

	function removeItem(index: number) {
		items = items.filter((_, i) => i !== index);
	}

	function searchModalClose() {
		// Remove is-active from modal
		document.querySelector('.modal')!.classList.remove('is-active');
	}

	function searchModal(index: number) {
		// Add is-active to modal
		document.querySelector('.modal')!.classList.add('is-active');
	}

	$: total = items.reduce((total, item) => total + item.quantity * item.price, 0);

	let product_search: ProductSearch;
</script>


<PermissionGuard permissions={['ORDER_WRITE', 'PRODUCT_READ']}>
<ProductSearch bind:this={product_search} />

{#if !is_mobile}
	<section class="section">
		<div class="container">
			<h1 class="title">Invoice Maker</h1>

			<table class="table is-fullwidth">
				<thead>
					<tr>
						<th>Name</th>
						<th>Quantity</th>
						<th>Price</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each items as item, i}
						<tr>
							<td
								><input
									class="input"
									type="text"
									bind:value={item.name}
									on:focus={(event) => {
                                        //@ts-ignore
                                        let res = product_search.search(event.target.value);

                                        if (res == null) {
                                            
                                        } else {
                                            //@ts-ignore
                                            event.target.value = res;
                                        }

                                        //@ts-ignore
                                        event.target.blur();
									}}
								/>
							</td><td><input class="input" type="number" bind:value={item.quantity} /></td>
							<td><input class="input" type="number" bind:value={item.price} /></td>
							<td
								><button class="button is-danger" on:click={() => removeItem(i)}>
									<i class="fas fa-trash"></i>
								</button></td
							>
						</tr>
					{/each}
				</tbody>
			</table>

			<button class="button is-primary" on:click={addItem}>Add Item</button>

			<h2 class="title">Total: ${total}</h2>
		</div>
	</section>
{:else}
	<div class="notification is-warning">Invoice maker UI not available on mobile.</div>
{/if}

<div slot="denied">
    <h1 class="title">You are not authorized to view this page</h1>
</div>
</PermissionGuard>
