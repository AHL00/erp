#![feature(lazy_cell)]

use db::DbConn;
use rocket::{config::SecretKey, fs::FileServer, Config};
use rocket_db_pools::Database;

pub mod permissions;
pub mod env;
pub mod db;
pub mod routes;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set env logger to info mode
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    let mut config = Config::from(Config::figment());

    config.port = *env::PORT;
    config.secret_key = SecretKey::from(&env::JWT_SECRET);
    
    rocket::Config {
        address: env::HOST.as_str().parse().unwrap(),
        port: *env::PORT,
        secret_key: SecretKey::from(&env::JWT_SECRET),
        ..Default::default()
    };

    // Note: To test a completely new DB, use the following: DROP SCHEMA public CASCADE;

    rocket::build()
        .mount("/api", routes::routes())
        .mount("/", FileServer::from("./frontend/dist").rank(3))
        .attach(rocket_cors::CorsOptions::default().to_cors().unwrap())
        .attach(DbConn::init())
        .launch()
        .await?;

    Ok(())
}
