<script lang="ts">
	import { logout } from '$lib/auth';
	import { redirect } from '$lib';
	import type { ListRequest } from '$bindings/ListRequest';
    import type { OrderMeta } from '$bindings/OrderMeta';
	import type { CrudColumn } from '../../../components/crud/types';
	import type { Customer } from '$bindings/Customer';
	import type { User } from '$bindings/User';
	import CrudPanel from '../../../components/crud/CrudPanel.svelte';
	import { list } from 'postcss';

    let order_list_req: ListRequest = {
		range: {
			count: 25,
			offset: 0
		},
		filters: [],
		sorts: [
            {
                column: 'date_time',
                order: 'DESC'
            }
        ]
	};

    let orders_list: OrderMeta[] = [];

    // Won't be editing so no need for edit config in columns
    let columns: CrudColumn[] = [
        {
            api_name: 'id',
            display_name: 'ID',
            display_map_fn: null,
            current_sort: null,
            edit_type: { type: 'none' },
            edit_readonly: true
        },
        {
            api_name: 'date_time',
            display_name: 'Date',
            display_map_fn: (val: string) => {
                let date = new Date(val);
                return date.toLocaleString();
            },
            current_sort: 'DESC',
            edit_type: { type: 'none' },
            edit_readonly: true
        },
        {
            api_name: 'customer',
            display_name: 'Customer',
            display_map_fn: (val: Customer) => {
                return val.name;
            },
            current_sort: null,
            edit_type: { type: 'none' },
            edit_readonly: true
        },
        {
            api_name: 'created_by_user',
            display_name: 'Created by',
            display_map_fn: (val: User) => {
                return val.username;
            },
            current_sort: null,
            edit_type: { type: 'none' },
            edit_readonly: true
        },
    ]
</script>

<div class="flex flex-col h-full w-full items-center overflow-hidden p-3">
	<div class="h-64 rounded-lg shadow-md bg-custom-bg-lighter dark:bg-custom-bg-dark mb-3">
		<span class="text-2xl">Create</span>
	</div>
    <div class="w-full rounded-lg p-1 h-full shadow-md bg-custom-bg-lighter dark:bg-custom-bg-dark flex flex-col">
        <span class="text-2xl m-3">Past orders</span>
        <CrudPanel
            list_request={order_list_req}
            objects_list={orders_list}
            crud_endpoint="orders"
            read_perms={['ORDER_READ']}
            write_perms={['ORDER_WRITE']}
            create_default={null}
            edit_override={
            (item_id) => {
                redirect(`/app/orders/${item_id}`);
            }}
            {columns}
            >
        </CrudPanel>
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
