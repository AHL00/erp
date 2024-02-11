<script lang="ts">
    import { redirect } from '$lib';
import { logout, auth_info_store } from '$lib/auth';
</script>

<div class="sidebar">
	<div class="item" on:click={() => redirect('/app/orders')}>
        <div class="icon">
            <i class="fa fa-clipboard-list"></i>
        </div>
		<span class="label">Orders</span>
    </div>
	
    <div class="separator"></div>
    
    <div class="item">
        
    </div>


	<div class="item" style="margin-top: auto;" on:click={logout}>
        <div class="icon">
            <i class="fa fa-user"></i>
        </div>
		<span class="label">
            Account
        </span>
	</div>
</div>

<style>
	.sidebar {
		width: 60px;
		height: 100vh;
		background-color: #ffffff;
		transition: width 0.2s ease-in-out;
		overflow: hidden;
		align-self: start;
		justify-self: start;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding-top: 10px;
		padding-bottom: 10px;
		box-shadow: 0 0 10px 0 rgba(0, 0, 0, 0.1);
	}

	.sidebar:hover {
		width: 200px;
	}

    .sidebar:hover > .item > .label {
        opacity: 1;
        transition: all 0.35s;
        width: 100%;
    }

    .sidebar > .item > .label {
        color: #444;
		transition: all 0.2s;
		opacity: 0;
		font-size: 18px;
		width: 0;
        align-self: center;
        text-align: center;
    }

    .sidebar:hover > .item > .icon {
        opacity: 0;
        transition: all 0.2s;
        width: 0;
    }

    .sidebar > .item > .icon {
        color: #333333b4;
        opacity: 1;
        transition: all 0.35s;
        width: 100%;
    }

	.item {
        width: 100%;
        height: 60px;
		font-size: 28px;
		cursor: pointer;
		transition: all 0.15s;
		display: flex;
		flex-direction: row;
		align-items: center;
	}

    .separator {
        height: 2px;
        margin-top: 9px;
        margin-bottom: 9px;
    }

	.item:hover {
		scale: 1.1;
	}
</style>
