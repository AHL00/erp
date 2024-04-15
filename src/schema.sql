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
  phone VARCHAR(255),
  address TEXT,
  notes TEXT
);

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
  price NUMERIC(10, 2) NOT NULL,
  stock INTEGER NOT NULL,
  quantity_per_box INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS orders (
  id SERIAL PRIMARY KEY,
  date_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
  customer_id INT NOT NULL,
  created_by_user_id INT NOT NULL,
  amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
  notes TEXT,
  FOREIGN KEY (customer_id) REFERENCES customers(id),
  FOREIGN KEY (created_by_user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS retail_orders (
  id SERIAL PRIMARY KEY,
  customer_name VARCHAR(255) NOT NULL,
  customer_phone VARCHAR(255), 
  order_id INT NOT NULL,
  created_by_user_id INT NOT NULL,
  FOREIGN KEY (order_id) REFERENCES orders(id)
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

-- Junction table for many-to-many relationship between retail orders and order items
CREATE TABLE IF NOT EXISTS retail_order_items (
  id SERIAL PRIMARY KEY,
  inventory_id INT NOT NULL,
  quantity INT NOT NULL,
  retail_order_id INT NOT NULL,
  FOREIGN KEY (retail_order_id) REFERENCES retail_orders(id),
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

CREATE OR REPLACE FUNCTION get_order(order_id INT) RETURNS JSON AS $$
DECLARE
  order_record RECORD;
  order_items RECORD;
  order_items_array JSON[];
    order_json JSON;
BEGIN
    SELECT * INTO order_record FROM orders WHERE id = order_id;
    SELECT * INTO order_items FROM order_items WHERE order_id = order_id;
    FOR order_items IN SELECT * FROM order_items WHERE order_id = order_id
    LOOP
        order_items_array = array_append(order_items_array, json_build_object('product', order_items.product, 'quantity', order_items.quantity));
    END LOOP;
    order_json = json_build_object('order', json_build_object('id', order_record.id, 'date_time', order_record.date_time, 'customer_id', order_record.customer_id, 'created_by_user_id', order_record.created_by_user_id, 'order_items', order_items_array));
    RETURN order_json;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_retail_order(retail_order_id INT) RETURNS JSON AS $$
DECLARE
  retail_order_record RECORD;
  retail_order_items RECORD;
  retail_order_items_array JSON[];
    retail_order_json JSON;
BEGIN
    SELECT * INTO retail_order_record FROM retail_orders WHERE id = retail_order_id;
    SELECT * INTO retail_order_items FROM retail_order_items WHERE retail_order_id = retail_order_id;
    FOR retail_order_items IN SELECT * FROM retail_order_items WHERE retail_order_id = retail_order_id
    LOOP
        retail_order_items_array = array_append(retail_order_items_array, json_build_object('product', retail_order_items.product, 'quantity', retail_order_items.quantity));
    END LOOP;
    retail_order_json = json_build_object('retail_order', json_build_object('id', retail_order_record.id, 'customer_name', retail_order_record.customer_name, 'customer_phone', retail_order_record.customer_phone, 'order_id', retail_order_record.order_id, 'created_by_user_id', retail_order_record.created_by_user_id, 'retail_order_items', retail_order_items_array));
    RETURN retail_order_json;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_purchase(purchase_id INT) RETURNS JSON AS $$
DECLARE
  purchase_record RECORD;
  purchase_items RECORD;
  purchase_items_array JSON[];
    purchase_json JSON;
BEGIN
    SELECT * INTO purchase_record FROM purchases WHERE id = purchase_id;
    SELECT * INTO purchase_items FROM purchase_items WHERE purchase_id = purchase_id;
    FOR purchase_items IN SELECT * FROM purchase_items WHERE purchase_id = purchase_id
    LOOP
        purchase_items_array = array_append(purchase_items_array, json_build_object('product', purchase_items.product, 'quantity', purchase_items.quantity));
    END LOOP;
    purchase_json = json_build_object('purchase', json_build_object('id', purchase_record.id, 'date_time', purchase_record.date_time, 'supplier_id', purchase_record.supplier_id, 'created_by_user_id', purchase_record.created_by_user_id, 'purchase_items', purchase_items_array));
    RETURN purchase_json;
    END;
$$ LANGUAGE plpgsql;

