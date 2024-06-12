CREATE SCHEMA IF NOT EXISTS public;

CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(255) UNIQUE NOT NULL,
  password VARCHAR(255) NOT NULL,
  salt VARCHAR(255) NOT NULL,
  permissions INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS customers (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255),
  name_ts tsvector GENERATED ALWAYS AS (to_tsvector('english', name)) STORED,
  phone VARCHAR(255),
  address TEXT,
  notes TEXT
);

CREATE INDEX IF NOT EXISTS customers_name_ts_idx ON customers USING GIN (name_ts);

CREATE TABLE IF NOT EXISTS suppliers (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255),
  phone VARCHAR(255),
  address TEXT,
  notes TEXT
);

CREATE TABLE IF NOT EXISTS inventory (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  name_ts tsvector GENERATED ALWAYS AS (to_tsvector('english', name)) STORED,
  price NUMERIC(10, 2) NOT NULL,
  stock INTEGER NOT NULL,
  quantity_per_box INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS inventory_name_ts_idx ON inventory USING GIN (name_ts);

CREATE TABLE IF NOT EXISTS orders (
  id SERIAL PRIMARY KEY,
  date_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
  customer_id INT NOT NULL,
  created_by_user_id INT NOT NULL,
  amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
  retail BOOLEAN NOT NULL DEFAULT FALSE,
  notes TEXT,
  FOREIGN KEY (customer_id) REFERENCES customers(id),
  FOREIGN KEY (created_by_user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS purchases (
  id SERIAL PRIMARY KEY,
  date_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
  supplier_id INT NOT NULL,
  created_by_user_id INT NOT NULL,
  amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
  FOREIGN KEY (supplier_id) REFERENCES suppliers(id),
  FOREIGN KEY (created_by_user_id) REFERENCES users(id)
);

-- Junction table for one-to-many relationship between orders and order items
CREATE TABLE IF NOT EXISTS order_items (
  id SERIAL PRIMARY KEY,
  inventory_id INT NOT NULL,
  price NUMERIC(10, 2) NOT NULL, -- price at the time of order, which can be edited freely to allow for discounts, special rates, etc.
  quantity INT NOT NULL,
  order_id INT NOT NULL,
  FOREIGN KEY (order_id) REFERENCES orders(id),
  FOREIGN KEY (inventory_id) REFERENCES inventory(id)
);

-- Junction table for many-to-many relationship between purchases and purchase items
CREATE TABLE IF NOT EXISTS purchase_items (
  id SERIAL PRIMARY KEY,
  inventory_id INT NOT NULL,
  quantity INT NOT NULL,
  price NUMERIC(10, 2) NOT NULL,
  purchase_id INT NOT NULL,
  FOREIGN KEY (purchase_id) REFERENCES purchases(id),
  FOREIGN KEY (inventory_id) REFERENCES inventory(id)
);

CREATE TABLE IF NOT EXISTS expenses (
  id SERIAL PRIMARY KEY,
  date_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
  amount NUMERIC(10, 2) NOT NULL,
  description TEXT,
  created_by_user_id INT NOT NULL,
  FOREIGN KEY (created_by_user_id) REFERENCES users(id)
);

-- Inserts into:
-- #[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
-- pub(super) struct OrderMeta {
--     pub id: i32,
--     pub date_time: chrono::DateTime<chrono::Utc>,
--     /// This will be false if the order is retail
--     pub customer: Option<Customer>, 
--     pub created_by_user: User,
--     pub amount_paid: BigDecimal,
--     pub retail: bool,
--     pub notes: String,
-- }

-- #[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
-- #[ts(export)]
-- pub(super) struct OrderItem {
--     pub id: i32,
--     pub inventory_item: InventoryItem,
--     pub quantity: i32,
--     pub price: BigDecimal,
-- }

-- #[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
-- #[ts(export)]
-- pub(super) struct Order {
--     #[serde(flatten)]
--     pub meta: OrderMeta,
--     pub items: Vec<OrderItem>,
-- }

CREATE OR REPLACE FUNCTION get_order_meta(order_id_input INT) RETURNS JSON AS $$
DECLARE
  order_record RECORD;
  created_by_user RECORD;
  customer RECORD;
  order_json JSON;
BEGIN
    SELECT * INTO order_record FROM orders WHERE id = order_id_input;

    -- if the order does not exist, raise an exception
    IF order_record IS NULL THEN
        RAISE EXCEPTION 'Order with id % not found', order_id_input;
    END IF;

    SELECT * INTO created_by_user FROM users WHERE id = order_record.created_by_user_id;
    SELECT * INTO customer FROM customers WHERE id = order_record.customer_id;

    order_json := json_build_object(
        'id', order_record.id,
        'date_time', order_record.date_time,
        'customer', CASE WHEN customer IS NULL THEN NULL ELSE json_build_object(
            'id', customer.id,
            'name', customer.name,
            'phone', customer.phone,
            'address', customer.address,
            'notes', customer.notes
        ) END,
        'created_by_user', json_build_object(
            'id', created_by_user.id,
            'username', created_by_user.username
        ),
        'amount_paid', order_record.amount_paid,
        'retail', order_record.retail,
        'notes', order_record.notes
    );

    RETURN order_json;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_order_items(order_id_input INT) RETURNS JSON AS $$
DECLARE
  order_items_array JSON[];
  order_items RECORD;
BEGIN
    -- if the order does not exist, raise an exception
    IF NOT EXISTS (SELECT 1 FROM orders WHERE id = order_id_input) THEN
        RAISE EXCEPTION 'Order with id % not found', order_id_input;
    END IF;

    FOR order_items IN SELECT * FROM order_items WHERE order_id = order_id_input
    LOOP
        order_items_array := array_append(order_items_array, json_build_object(
        'id', order_items.id,
        'inventory_item', (SELECT json_build_object(
            'id', inventory.id,
            'name', inventory.name,
            'price', inventory.price,
            'stock', inventory.stock,
            'quantity_per_box', inventory.quantity_per_box
        ) FROM inventory WHERE inventory.id = order_items.inventory_id),
        'quantity', order_items.quantity,
        'price', order_items.price
        ));
    END LOOP;
    RETURN array_to_json(order_items_array);
END;
$$ LANGUAGE plpgsql;


-- This eliminates the need for multiple queries to get the order details
-- from multiple tables. The function will return a JSON object with the
-- order details and the items in the order. This reduces the number of
-- round trips to the database.
CREATE OR REPLACE FUNCTION get_order(order_id_input INT) RETURNS JSON AS $$
DECLARE
  order_record RECORD;
  order_items RECORD;
    created_by_user RECORD;
    customer RECORD;
    order_json JSON;
BEGIN
    SELECT * INTO order_record FROM orders WHERE id = order_id_input;

    -- if the order does not exist, raise an exception
    IF order_record IS NULL THEN
        RAISE EXCEPTION 'Order with id % not found', order_id_input;
    END IF;

    SELECT * INTO created_by_user FROM users WHERE id = order_record.created_by_user_id;
    SELECT * INTO customer FROM customers WHERE id = order_record.customer_id;

    order_json := json_build_object(
        'meta', get_order_meta(order_id_input),
        'items', get_order_items(order_id_input)
    );

    RETURN order_json;
    END;
$$ LANGUAGE plpgsql;

