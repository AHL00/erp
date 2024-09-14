CREATE SCHEMA IF NOT EXISTS public;

CREATE TABLE
    IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(255) UNIQUE NOT NULL,
        password VARCHAR(255) NOT NULL,
        salt VARCHAR(255) NOT NULL,
        permissions INTEGER NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS customers (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255),
        name_ts tsvector GENERATED ALWAYS AS (to_tsvector ('english', name)) STORED,
        phone VARCHAR(255),
        address TEXT,
        notes TEXT
    );

CREATE INDEX IF NOT EXISTS customers_name_ts_idx ON customers USING GIN (name_ts);

CREATE TABLE
    IF NOT EXISTS suppliers (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255),
        phone VARCHAR(255),
        address TEXT,
        notes TEXT
    );

CREATE TABLE
    IF NOT EXISTS inventory (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        name_ts tsvector GENERATED ALWAYS AS (to_tsvector ('english', name)) STORED,
        price NUMERIC(10, 2) NOT NULL,
        stock INTEGER NOT NULL,
        quantity_per_box INTEGER NOT NULL
    );

CREATE INDEX IF NOT EXISTS inventory_name_ts_idx ON inventory USING GIN (name_ts);

CREATE TABLE
    IF NOT EXISTS orders (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            customer_id INT,
            created_by_user_id INT NOT NULL,
            amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
            retail BOOLEAN NOT NULL DEFAULT FALSE,
            retail_customer_name VARCHAR(255),
            retail_customer_phone VARCHAR(255),
            retail_customer_address TEXT,
            notes TEXT,
            fulfilled BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY (customer_id) REFERENCES customers (id),
            FOREIGN KEY (created_by_user_id) REFERENCES users (id)
    );

CREATE TABLE
    IF NOT EXISTS purchases (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            supplier_id INT NOT NULL,
            created_by_user_id INT NOT NULL,
            amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
            FOREIGN KEY (supplier_id) REFERENCES suppliers (id),
            FOREIGN KEY (created_by_user_id) REFERENCES users (id)
    );

-- Junction table for one-to-many relationship between orders and order items
CREATE TABLE
    IF NOT EXISTS order_items (
        id SERIAL PRIMARY KEY,
        inventory_id INT NOT NULL,
        price NUMERIC(10, 2) NOT NULL, -- price at the time of order, which can be edited freely to allow for discounts, special rates, etc.
        quantity INT NOT NULL,
        order_id INT NOT NULL,
        FOREIGN KEY (order_id) REFERENCES orders (id),
        FOREIGN KEY (inventory_id) REFERENCES inventory (id)
    );

-- Junction table for many-to-many relationship between purchases and purchase items
CREATE TABLE
    IF NOT EXISTS purchase_items (
        id SERIAL PRIMARY KEY,
        inventory_id INT NOT NULL,
        quantity INT NOT NULL,
        price NUMERIC(10, 2) NOT NULL,
        purchase_id INT NOT NULL,
        FOREIGN KEY (purchase_id) REFERENCES purchases (id),
        FOREIGN KEY (inventory_id) REFERENCES inventory (id)
    );

CREATE TABLE
    IF NOT EXISTS expenses (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            amount NUMERIC(10, 2) NOT NULL,
            description TEXT,
            description_ts tsvector GENERATED ALWAYS AS (to_tsvector ('english', description)) STORED,
            created_by_user_id INT NOT NULL,
            FOREIGN KEY (created_by_user_id) REFERENCES users (id)
    );

CREATE TABLE
    IF NOT EXISTS stock_updates (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            inventory_id INT NOT NULL,
            created_by_user_id INT NOT NULL,
            delta INTEGER NOT NULL,
            -- NOTE: All of these could be invalid if
            -- any of them are deleted. 
            order_item_id INT,
            order_id INT,
            purchase_item_id INT,
            purchase_id INT,
            FOREIGN KEY (created_by_user_id) REFERENCES users (id),
            FOREIGN KEY (inventory_id) REFERENCES inventory (id)
    );

CREATE TABLE
    IF NOT EXISTS settings (
        id SERIAL PRIMARY KEY,
        key TEXT UNIQUE NOT NULL,
        long_name TEXT UNIQUE NOT NULL,
        description TEXT,
        value JSONB NOT NULL
    );