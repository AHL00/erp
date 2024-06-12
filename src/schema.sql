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

CREATE OR REPLACE FUNCTION get_customer_json(customer_id_input INT) RETURNS JSON AS $$
DECLARE
  customer_record RECORD;
BEGIN
    SELECT * INTO customer_record FROM customers WHERE id = customer_id_input;

    -- if the customer does not exist, return NULL
    IF customer_record IS NULL THEN
        RETURN NULL;
    END IF;

    RETURN json_build_object(
        'id', customer_record.id,
        'name', customer_record.name,
        'phone', customer_record.phone,
        'address', customer_record.address,
        'notes', customer_record.notes
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_user_json(user_id_input INT) RETURNS JSON AS $$
DECLARE
  user_record RECORD;
BEGIN
    SELECT * INTO user_record FROM users WHERE id = user_id_input;

    -- if the user does not exist, return NULL
    IF user_record IS NULL THEN
        RETURN NULL;
    END IF;

    RETURN json_build_object(
        'id', user_record.id,
        'username', user_record.username,
        'password', user_record.password,
        'salt', user_record.salt,
        'permissions', user_record.permissions
    );
END;
$$ LANGUAGE plpgsql;
