
// Environment variables
// JWT_SECRET: Secret key for JWT signing
// PORT: Port to listen on
// HOST: Host to listen on
// DATABASE_PATH: Path to SQLite database

use std::sync::LazyLock;

pub static DATABASE_PASSWORD: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_PASSWORD")
        .unwrap_or_else(|_| {
            let password = "SYf4ctmI8unj4Rr8mSw7FJdULPL3AbEq".to_string();

            log::warn!("No DATABASE_PASSWORD environment variable found, using default password {}", password);

            password
        })
});

pub static PUBLIC_DIR: LazyLock<String> = LazyLock::new(|| {
    std::env::var("PUBLIC_DIR")
        .unwrap_or_else(|_| {
            log::warn!("No PUBLIC_DIR environment variable found, using default path frontend/dist");

            "frontend/dist".to_string()
        })
});

pub static PORT: LazyLock<u16> = LazyLock::new(|| {
    std::env::var("PORT")
        .unwrap_or_else(|_| {
            log::warn!("No PORT environment variable found, using default port 8080");

            "8080".to_string()
        })
        .parse()
        .expect("Failed to parse PORT environment variable")
});

pub static HOST: LazyLock<String> = LazyLock::new(|| {
    std::env::var("HOST")
        .unwrap_or_else(|_| {
            log::warn!("No HOST environment variable found, using default host localhost");

            "127.0.0.1".to_string()
        })
});