#![feature(lazy_cell)]

use std::{
    path::Path,
    sync::{LazyLock, RwLock},
};

use rocket::{config::SecretKey, fs::FileServer, routes};
use serde::{Deserialize, Serialize};

use crate::auth::UserPermissions;

pub mod auth;
pub mod env;
pub mod search;

#[derive(Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f32,
    pub stock: u32,
    pub quantity_per_box: u32,
}

pub static mut DB: LazyLock<RwLock<rusqlite::Connection>> = LazyLock::new(|| {
    if !std::path::Path::new(&env::DATABASE_PATH.as_str()).exists() {
        log::info!("Creating new db with schema");

        let conn = rusqlite::Connection::open("db.sqlite3").unwrap();

        conn.execute_batch(include_str!("schema.sql"))
            .expect("Failed to create schema");

        let admin_permissions = UserPermissions::ManageDB
            | UserPermissions::OrderRead
            | UserPermissions::OrderRead
            | UserPermissions::OrderWrite
            | UserPermissions::ProductRead
            | UserPermissions::ProductWrite
            | UserPermissions::UserRead
            | UserPermissions::UserWrite;

        // Default users
        conn.execute(
            "INSERT INTO users (username, password, permissions) VALUES (?, ?, ?)",
            ["admin", "admin", &(admin_permissions as u32).to_string()],
        )
        .unwrap();

        // Import from csv
        // Embed for now
        let csv = include_str!("../test_products.csv").split('\n');

        let count = csv
            .map(|line| {
                let mut split = line.split(',');
                let id = split.next().unwrap();
                let name = split.next().unwrap();
                let price = split.next().unwrap();
                let stock = split.next().unwrap();
                let quantity_per_box = split.next().unwrap();

                conn.execute(
                    "INSERT INTO products (id, name, price, stock, quantity_per_box) VALUES (?, ?, ?, ?, ?)",
                    [id, name, price, stock, quantity_per_box],
                )
            }).count();

        log::info!("Added {} test products", count);

        RwLock::new(conn)
    } else {
        log::info!("Using existing db");

        RwLock::new(rusqlite::Connection::open("db.sqlite3").unwrap())
    }
});

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set env logger to info mode
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    // Make sure lazy cell initializes
    let db = unsafe { DB.read().expect("Failed to get DB") };
    drop(db);

    let config = rocket::Config {
        address: env::HOST.as_str().parse().unwrap(),
        port: *env::PORT,
        secret_key: SecretKey::from(&env::JWT_SECRET),
        ..Default::default()
    };

    rocket::custom(config)
        .mount(
            "/api",
            routes![auth::login, auth::status, auth::logout, search::product],
        )
        .mount(
            "/",
            FileServer::from(Path::new(env::PUBLIC_DIR.as_str())).rank(3),
        )
        .attach(rocket_cors::CorsOptions::default().to_cors().unwrap())
        .launch()
        .await?;

    Ok(())
}

// #[async_std::main]
// async fn main() -> Result<(), std::io::Error> {
//     tide::log::start();

//     // Make sure lazy cell initializes
//     let _ = unsafe { DB.read() };

//     let mut app = tide::new();

//     // CORS middleware
//     app.with(tide::security::CorsMiddleware::new());

//     // Session middleware
//     app.with(
//         tide::sessions::SessionMiddleware::new(
//             tide::sessions::CookieStore::new(),
//             &env::JWT_SECRET.as_ref(),
//         )
//         .with_same_site_policy(tide::http::cookies::SameSite::None),
//     );

//     app.at("/api/auth/login").post(auth::tide_login);
//     app.at("/api/auth/info").get(auth::tide_info);
//     app.at("/api/auth/logout").post(auth::tide_logout);

//     app.at("/api/orders")
//         .with(UserPermissionsMiddleware::new(UserPermissions::Admin))
//         .get(|_| async { Ok(tide::Response::new(200)) });

//     app.listen(format!("{}:{}", *env::HOST, *env::PORT))
//         .await
//         .unwrap();

//     Ok(())
// }
