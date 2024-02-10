-- CREATE SCHEMA IF NOT EXISTS public;
CREATE TABLE
    IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username VARCHAR(255) UNIQUE NOT NULL,
        password VARCHAR(255) NOT NULL,
        /* 
        The permissions column is an integer that represents the user's permissions. 
        We can use bitwise operations to check if a user has a specific permission.
         */
        permissions INT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS customers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name VARCHAR(255) NOT NULL,
        phone VARCHAR(255) NOT NULL,
        address VARCHAR(255) NOT NULL,
        delivery_method VARCHAR(255) NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS products (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name VARCHAR(255) NOT NULL,
        price DECIMAL(10, 2) NOT NULL,
        stock INT NOT NULL,
        quantity_per_box INT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS orders (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TIMESTAMP NOT NULL,
        customer INT NOT NULL,
        created_by_user INT NOT NULL,
        FOREIGN KEY (customer) REFERENCES customers (id),
        FOREIGN KEY (created_by_user) REFERENCES users (id)
    );

/* 
The order_items table is a junction table that connects the orders and products tables. 
It contains the foreign keys of both tables and a quantity column that represents the number of products ordered.
 */
CREATE TABLE
    IF NOT EXISTS order_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        product INT NOT NULL,
        -- order is a reserved keyword in SQL, so we use order_ instead
        order_ INT NOT NULL,
        quantity INT NOT NULL,
        FOREIGN KEY (order_) REFERENCES orders (id),
        FOREIGN KEY (product) REFERENCES products (id)
    );