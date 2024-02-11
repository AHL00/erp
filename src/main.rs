#![feature(lazy_cell)]

use std::sync::LazyLock;

use pollster::FutureExt;
use rocket::{config::SecretKey, fs::FileServer, routes};

use tokio_postgres::{tls::NoTlsStream, GenericClient, NoTls, Socket};

pub mod auth;
pub mod env;

pub struct DbState {
    pub client: tokio_postgres::Client,
    pub connection: tokio_postgres::Connection<Socket, NoTlsStream>,
}

unsafe impl Send for DbState {}
unsafe impl Sync for DbState {}

pub static mut DB: LazyLock<tokio::sync::RwLock<DbState>> = LazyLock::new(|| {
    log::info!("Connecting to database...");

    let (client, connection) = tokio_postgres::connect(
        format!(
            "host={} user={} port={} dbname={} password={}",
            *env::DATABASE_HOST,
            *env::DATABASE_USER,
            *env::DATABASE_PORT,
            *env::DATABASE_NAME,
            *env::DATABASE_PASSWORD
        )
        .as_str(),
        NoTls,
    )
    .block_on()
    .unwrap_or_else(|e| {
        log::error!("Failed to connect to database: {}", e);
        std::process::exit(1);
    });

    log::info!("Connected to database");

    // Make sure schema is created
    // client
    //     .batch_execute(include_str!("schema.sql"))
    //     .block_on()
    //     .unwrap_or_else(|e| {
    //         log::error!("Failed to create schema: {}", e);
    //         std::process::exit(1);
    //     });

    client
        .execute("CREATE SCHEMA IF NOT EXISTS public", &[])
        .block_on()
        .unwrap();

    log::info!("Schema created");

    tokio::sync::RwLock::new(DbState { client, connection })
});

#[tokio::main]
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
