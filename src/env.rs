
// Environment variables
// JWT_SECRET: Secret key for JWT signing
// PORT: Port to listen on
// HOST: Host to listen on
// DATABASE_PATH: Path to SQLite database

use std::sync::LazyLock;

pub static DATABASE_PATH: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| {
            log::warn!("No DATABASE_PATH environment variable found, using default path db.sqlite3");

            "db.sqlite3".to_string()
        })
});

pub static JWT_SECRET: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_|
        {
            log::warn!("No JWT_SECRET environment variable found, using default secret");

            "default_secret_lashflashlalkshjsakl".to_string()
        });

    // Make sure secret is a valid length
    if secret.len() < 32 {
        log::error!("JWT_SECRET must be at least 32 bytes long");
        std::process::exit(1);
    }

    secret.as_bytes().to_vec()
});

pub static PORT: LazyLock<u16> = LazyLock::new(|| {
    std::env::var("PORT")
        .unwrap_or_else(|_| {
            log::warn!("No PORT environment variable found, using default port 8000");

            "8000".to_string()
        })
        .parse()
        .expect("Failed to parse PORT environment variable")
});

pub static HOST: LazyLock<String> = LazyLock::new(|| {
    std::env::var("HOST")
        .unwrap_or_else(|_| {
            log::warn!("No HOST environment variable found, using default host 0.0.0.0");

            "0.0.0.0".to_string()
        })
});