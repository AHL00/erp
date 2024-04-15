<!-- This page is for editing existing orders. It assumes the order exists on the server. -->

<script lang="ts">
    let url_params = new URLSearchParams(window.location.search);

    // Make sure order id is valid
    let order_id: number;
    try {
        let id_str = url_params.get('id');

        if (id_str == null) {
            throw new Error('No order ID provided');
        }

        order_id = parseInt(id_str);

        if (isNaN(order_id)) {
            throw new Error('Invalid order ID provided');
        }
    } catch (e) {
        console.error(e);
        window.location.href = '/app/orders';
    }

    let order = null;

    let loading_count = 0;

    let initial_order = null;
    
    function load_order() {
        loading_count++;

        fetch(`/api/orders/${order_id}`)
            .then((res) => {
                if (res.status == 200) {
                    res.json().then((data) => {
                        order = data;
                    });

                    if (initial_order == null) {
                        initial_order = order;
                    }

                } else {
                    console.error('Error fetching order');
                }
                loading_count--;
            });
    }

    load_order();

</script>



