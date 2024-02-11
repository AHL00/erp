<div
	class="box"
	style="height: 100%; width: 100%; display: flex; justify-content: center; align-items: center;"
>
	<div class="loader-wrapper">
		<div class="loader is-loading"></div>
	</div>
</div>

<style>
	.loader-wrapper {
		position: absolute;
		top: 0;
		left: 0;
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
