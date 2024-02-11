#![feature(lazy_cell)]

use std::sync::LazyLock;

use rocket::{config::SecretKey, fs::FileServer, routes};

use crate::auth::UserPermissions;

pub mod auth;
pub mod env;

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

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set env logger to info mode
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();


    // Make sure lazy cell initializes
    let _ = unsafe { DB.read() };

    let config = rocket::Config {
        address: env::HOST.as_str().parse().unwrap(),
        port: *env::PORT,
        secret_key: SecretKey::from(&env::JWT_SECRET),
        ..Default::default()
    };

    rocket::custom(config)
        .mount("/api", routes![auth::login, auth::status, auth::logout])
        .mount("/", FileServer::from("./frontend/dist").rank(3))
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
