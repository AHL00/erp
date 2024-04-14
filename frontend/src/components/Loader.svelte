<div
	class="box"
	style="height: 100%; width: 100%; display: flex; justify-content: center; align-items: center; box-shadow: none !important; z-index: 1000; position: fixed; top: 0; left: 0; background-color: rgba(255, 255, 255, 0.5);"
>
	<div class="loader-wrapper">
		<div class="loader is-loading"></div>
	</div>
</div>

TODO Loader broken

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
