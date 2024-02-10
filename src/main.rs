#![feature(lazy_cell)]

use std::{ops::BitOr, sync::LazyLock};

pub mod auth;
pub mod env;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum UserPermissions {
    ProductRead = 0b00000001,
    ProductWrite = 0b00000010,
    OrderRead = 0b00000100,
    OrderWrite = 0b00001000,
    UserRead = 0b00010000,
    UserWrite = 0b00100000,
    Admin = 0b10000000,
}

impl BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u8 | rhs as u8) }
    }
}

pub static mut DB: LazyLock<async_std::sync::RwLock<rusqlite::Connection>> = LazyLock::new(|| {
    if !std::path::Path::new(&env::DATABASE_PATH.as_str()).exists() {
        log::info!("Creating new db with schema");

        let conn = rusqlite::Connection::open("db.sqlite3").unwrap();

        conn.execute_batch(include_str!("schema.sql"))
            .expect("Failed to create schema");

        let admin_permissions = UserPermissions::Admin;

        // Default users
        conn.execute(
            "INSERT INTO users (username, password, permissions) VALUES (?, ?, ?)",
            ["admin", "admin", &(admin_permissions as u8).to_string()],
        )
        .unwrap();

        async_std::sync::RwLock::new(conn)
    } else {
        log::info!("Using existing db");

        async_std::sync::RwLock::new(rusqlite::Connection::open("db.sqlite3").unwrap())
    }
});

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();

    // Make sure lazy cell initializes
    let _ = unsafe { DB.read() };

    let mut app = tide::new();

    // Session middleware
    app.with(tide::sessions::SessionMiddleware::new(
        tide::sessions::CookieStore::new(),
        &env::JWT_SECRET.as_ref(),
    ));

    // Serve Svelte app
    app.at("/").serve_dir("public/")?;
    app.at("/").serve_file("public/index.html")?;

    app.at("/api/auth/login").post(auth::login);
    app.at("/api/auth/info").get(auth::info);
    app.at("/api/auth/logout").post(auth::logout);

    app.listen(format!("{}:{}", *env::HOST, *env::PORT))
        .await
        .unwrap();

    Ok(())
}
