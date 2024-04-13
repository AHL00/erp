use std::{net::IpAddr, str::FromStr};

use db::DatabaseConnection;
use rocket::{fs::FileServer, Config};
use rocket_db_pools::Database;

pub mod db;
pub mod env;
pub mod routes;
pub mod types;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set env logger to info mode
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    // env file
    dotenv::dotenv().ok();

    let mut config = Config::from(Config::figment());

    config.port = env::port();
    config.address = IpAddr::from_str(&env::host()).expect("Failed to parse IP address");

    // Note: To test a completely new DB, use the following: DROP SCHEMA public CASCADE;

    let rocket = rocket::build()
        .mount("/api", routes::routes())
        .attach(rocket_cors::CorsOptions::default().to_cors().unwrap())
        .attach(DatabaseConnection::init());

    #[cfg(debug_assertions)]
    let rocket = rocket.mount("/", FileServer::from(env::public_dir()));

    rocket.launch().await?;

    Ok(())
}
