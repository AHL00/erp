<script lang="ts">
	import { logout } from '$lib/auth';
    import { redirect } from '$lib';
</script>

<div
	style="display: flex; flex-direction: row; overflow: hidden; height: 100vh; align-items: center; justify-content: center;"
>
	<div
		class="tile is-ancestor"
		style="justify-content: center; align-items: center; height: 100vh; max-width: 1000px;"
	>
		<div class="tile is-parent" on:click={() => redirect('/app/orders/create')}>
			<article class="tile is-child notification m-5 button-box" style="height: 40vh">
				<span>New Order</span>
			</article>
		</div>

		<div class="tile is-parent" on:click={() => redirect('/app/orders/view')}>
			<article class="tile is-child notification m-5 button-box" style="height: 40vh">
				<span>View Orders</span>
			</article>
		</div>
	</div>
</div>

<style>
	.button-box {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}

	.button-box:hover {
		background-color: #f0f0f0;
	}

	.button-box i {
		font-size: 3em;
	}

	.button-box span {
		font-size: 1.5em;
	}
</style>
