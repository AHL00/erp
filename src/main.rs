use db::DatabaseConnection;
use rocket_db_pools::Database;

pub mod db;
pub mod env;
pub mod settings;
pub mod routes;

pub mod types;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set env logger to info mode
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    dotenv::dotenv().ok();

    // Note: To reset the DB for testing, use the following SQL code: DROP SCHEMA public CASCADE;

    let cors_options = rocket_cors::CorsOptions {
        allow_credentials: true,
        ..Default::default()
    };

    let rocket = rocket::build()
        .mount(env::api_root(), routes::routes())
        .mount(
            format!("{}/public", env::api_root()),
            routes::public::routes(),
        )
        .attach(cors_options.to_cors().unwrap())
        .attach(DatabaseConnection::init());
    
    let rocket = if env::public_dir().len() > 0 {
        log::info!("Serving static files from: {}", env::public_dir());
        rocket.mount("/", rocket::fs::FileServer::from(env::public_dir()))
    } else {
        rocket
    };

    rocket.launch().await?;

    Ok(())
}
