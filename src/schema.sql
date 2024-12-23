CREATE SCHEMA IF NOT EXISTS public;

CREATE EXTENSION IF NOT EXISTS pg_trgm;

SELECT set_limit(0.1);

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
        phone VARCHAR(255),
        address TEXT,
        notes TEXT
    );

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
        description TEXT NOT NULL,
        price NUMERIC(32, 4) NOT NULL,
        stock INTEGER NOT NULL,
        quantity_per_box INTEGER NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS orders (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            customer_id INT,
            created_by_user_id INT NOT NULL,
            amount_paid NUMERIC(32, 4) NOT NULL DEFAULT 0.00,
            retail BOOLEAN NOT NULL DEFAULT FALSE,
            retail_customer_name VARCHAR(255),
            retail_customer_phone VARCHAR(255),
            retail_customer_address TEXT,
            notes TEXT,
            fulfilled BOOLEAN NOT NULL DEFAULT FALSE,
            -- Order total that updates automatically
            total NUMERIC(32, 4) NOT NULL DEFAULT 0.00,
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
            amount_paid NUMERIC(32, 4) NOT NULL DEFAULT 0.00,
            notes TEXT,
            FOREIGN KEY (supplier_id) REFERENCES suppliers (id),
            FOREIGN KEY (created_by_user_id) REFERENCES users (id)
    );

-- Junction table for one-to-many relationship between orders and order items
CREATE TABLE
    IF NOT EXISTS order_items (
        id SERIAL PRIMARY KEY,
        inventory_id INT NOT NULL,
        price NUMERIC(32, 4) NOT NULL, -- price at the time of order, which can be edited freely to allow for discounts, special rates, etc.
        quantity INT NOT NULL,
        order_id INT NOT NULL,
        discount NUMERIC(32, 4) NOT NULL DEFAULT 0.00,
        discount_percentage BOOLEAN NOT NULL DEFAULT FALSE,
        FOREIGN KEY (order_id) REFERENCES orders (id),
        FOREIGN KEY (inventory_id) REFERENCES inventory (id)
    );


DROP FUNCTION IF EXISTS get_order_total(INT);

-- NOTE: If updating this, make sure to change the function
-- in index.ts as well
CREATE OR REPLACE FUNCTION get_order_total(o_id INT) RETURNS NUMERIC AS $$
DECLARE
    total NUMERIC(32, 4);
BEGIN
    SELECT COALESCE(SUM(
        CASE
            WHEN discount_percentage THEN price * quantity * (1 - discount / 100)
            ELSE price * quantity - discount
        END
    ), 0)
    INTO total
    FROM order_items
    WHERE order_items.order_id = o_id;

    RETURN total;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_order_total_insert ON order_items;
DROP TRIGGER IF EXISTS update_order_total_update ON order_items;
DROP TRIGGER IF EXISTS update_order_total_delete ON order_items;
DROP FUNCTION IF EXISTS update_order_total();

-- Triggers to update the total of an order when an order item is inserted, updated, or deleted
CREATE OR REPLACE FUNCTION update_order_total() RETURNS TRIGGER AS $$
BEGIN
    UPDATE orders
    SET total = get_order_total(NEW.order_id)
    WHERE id = NEW.order_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_order_total_insert
AFTER INSERT ON order_items
FOR EACH ROW
EXECUTE FUNCTION update_order_total();

CREATE TRIGGER update_order_total_update
AFTER UPDATE ON order_items
FOR EACH ROW
EXECUTE FUNCTION update_order_total();

CREATE TRIGGER update_order_total_delete
AFTER DELETE ON order_items
FOR EACH ROW
EXECUTE FUNCTION update_order_total();

-- Create triggers if they do not exist
-- DO $$
-- BEGIN
--     IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_order_total_insert') THEN
--         CREATE TRIGGER update_order_total_insert
--         AFTER INSERT ON order_items
--         FOR EACH ROW
--         EXECUTE FUNCTION update_order_total();
--     END IF;

--     IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_order_total_update') THEN
--         CREATE TRIGGER update_order_total_update
--         AFTER UPDATE ON order_items
--         FOR EACH ROW
--         EXECUTE FUNCTION update_order_total();
--     END IF;

--     IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_order_total_delete') THEN
--         CREATE TRIGGER update_order_total_delete
--         AFTER DELETE ON order_items
--         FOR EACH ROW
--         EXECUTE FUNCTION update_order_total();
--     END IF;
-- END $$;

-- Junction table for many-to-many relationship between purchases and purchase items
CREATE TABLE
    IF NOT EXISTS purchase_items (
        id SERIAL PRIMARY KEY,
        inventory_id INT NOT NULL,
        price NUMERIC(32, 4) NOT NULL, -- price at the time of purchase, which should not be edited
        quantity INT NOT NULL,
        purchase_id INT NOT NULL,
        FOREIGN KEY (purchase_id) REFERENCES purchases (id),
        FOREIGN KEY (inventory_id) REFERENCES inventory (id)
    );

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'payment_method_t') THEN
        CREATE TYPE payment_method_t AS ENUM ('CASH', 'BANK', 'MOBILE', 'OTHER');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'party_type_t') THEN
        CREATE TYPE party_type_t AS ENUM ('CUSTOMER', 'SUPPLIER', 'RETAIL');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transfer_type_t') THEN
        CREATE TYPE transfer_type_t AS ENUM ('INCOMING', 'OUTGOING');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS payments (
    id SERIAL PRIMARY KEY,
    date_time TIMESTAMP
    WITH
        TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
        amount NUMERIC(32, 4) NOT NULL,
        party_id INT,
        party_type party_type_t NOT NULL,
        transfer_type transfer_type_t NOT NULL,
        method payment_method_t NOT NULL,
        method_details TEXT,
        notes TEXT,
        created_by_user_id INT NOT NULL,
        FOREIGN KEY (created_by_user_id) REFERENCES users (id)
);

-- Function to enforce foreign key constraint based on party_type
CREATE OR REPLACE FUNCTION enforce_party_fk()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.party_type = 'CUSTOMER' THEN
        -- Ensure party_id references customers table
        IF NOT EXISTS (SELECT 1 FROM customers WHERE id = NEW.party_id) THEN
            RAISE EXCEPTION 'Invalid party_id for CUSTOMER';
        END IF;
    ELSIF NEW.party_type = 'RETAIL' THEN
        -- Ensure party_id references orders table
        IF NOT EXISTS (SELECT 1 FROM orders WHERE id = NEW.party_id) THEN
            RAISE EXCEPTION 'Invalid party_id for RETAIL, expected a valid order_id';
        END IF;
    ELSIF NEW.party_type = 'SUPPLIER' THEN
        -- Ensure party_id references suppliers table
        IF NOT EXISTS (SELECT 1 FROM suppliers WHERE id = NEW.party_id) THEN
            RAISE EXCEPTION 'Invalid party_id for SUPPLIER';
        END IF;
    ELSE
        RAISE EXCEPTION 'Invalid party_type';
    END IF;

    -- Handle updates to party_type
    IF TG_OP = 'UPDATE' AND OLD.party_type <> NEW.party_type THEN
        IF NEW.party_type = 'CUSTOMER' THEN
            -- Ensure new party_id references customers table
            IF NOT EXISTS (SELECT 1 FROM customers WHERE id = NEW.party_id) THEN
                RAISE EXCEPTION 'Invalid party_id for CUSTOMER';
            END IF;
        ELSIF NEW.party_type = 'RETAIL' THEN
            -- Ensure new party_id references orders table
            IF NOT EXISTS (SELECT 1 FROM orders WHERE id = NEW.party_id) THEN
                RAISE EXCEPTION 'Invalid party_id for RETAIL, expected a valid order_id';
            END IF;
        ELSIF NEW.party_type = 'SUPPLIER' THEN
            -- Ensure new party_id references suppliers table
            IF NOT EXISTS (SELECT 1 FROM suppliers WHERE id = NEW.party_id) THEN
                RAISE EXCEPTION 'Invalid party_id for SUPPLIER';
            END IF;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function before insert or update
DROP TRIGGER IF EXISTS enforce_party_fk_trigger ON payments;
CREATE TRIGGER enforce_party_fk_trigger 
BEFORE INSERT OR UPDATE ON payments
FOR EACH ROW EXECUTE FUNCTION enforce_party_fk();

-- Function to prevent deletion of referenced rows
CREATE OR REPLACE FUNCTION prevent_deletion_if_referenced()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM payments WHERE party_id = OLD.id) THEN
        IF TG_TABLE_NAME = 'customers' AND OLD.id IN (SELECT party_id FROM payments WHERE party_type = 'CUSTOMER') THEN
            RAISE EXCEPTION 'Cannot delete customer with id % because it is referenced in payments', OLD.id;
        END IF;
        
        IF TG_TABLE_NAME = 'suppliers' AND OLD.id IN (SELECT party_id FROM payments WHERE party_type = 'SUPPLIER') THEN
            RAISE EXCEPTION 'Cannot delete supplier with id % because it is referenced in payments', OLD.id;
        END IF;

        IF TG_TABLE_NAME = 'orders' AND OLD.id IN (SELECT party_id FROM payments WHERE party_type = 'RETAIL') THEN
            RAISE EXCEPTION 'Cannot delete order with id % because it is referenced in payments', OLD.id;
        END IF;
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for customers, suppliers, and orders tables
DROP TRIGGER IF EXISTS prevent_customer_deletion ON customers;
CREATE TRIGGER prevent_customer_deletion
BEFORE DELETE ON customers
FOR EACH ROW
EXECUTE FUNCTION prevent_deletion_if_referenced();

DROP TRIGGER IF EXISTS prevent_supplier_deletion ON suppliers;
CREATE TRIGGER prevent_supplier_deletion
BEFORE DELETE ON suppliers
FOR EACH ROW
EXECUTE FUNCTION prevent_deletion_if_referenced();

DROP TRIGGER IF EXISTS prevent_order_deletion ON orders;
CREATE TRIGGER prevent_order_deletion
BEFORE DELETE ON orders
FOR EACH ROW
EXECUTE FUNCTION prevent_deletion_if_referenced();

CREATE TABLE
    IF NOT EXISTS expenses (
        id SERIAL PRIMARY KEY,
        date_time TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP, -- automatically set to current time
            amount NUMERIC(32, 4) NOT NULL,
            description TEXT,
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

-- CREATE OR REPLACE FUNCTION get_purchase_total(purchase_id INT) RETURNS NUMERIC AS $$
-- DECLARE
--     total NUMERIC(32, 4);
-- BEGIN
--     SELECT COALESCE(SUM(price * quantity), 0)
--     INTO total
--     FROM purchase_items
--     WHERE purchase_id = get_purchase_total.purchase_id;

--     RETURN total;
-- END;