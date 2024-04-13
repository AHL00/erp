
// Environment variables
// JWT_SECRET: Secret key for JWT signing
// PORT: Port to listen on
// HOST: Host to listen on
// DATABASE_PATH: Path to SQLite database

use std::sync::LazyLock;

pub static DATABASE_HOST: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_HOST")
        .unwrap_or_else(|_| {
            let host = "dpg-cn4bfoocmk4c73em0bug-a.singapore-postgres.render.com".to_string();

            log::warn!("No DATABASE_HOST environment variable found, using default host {}", host);

            host
        })
});

pub static DATABASE_PORT: LazyLock<u16> = LazyLock::new(|| {
    std::env::var("DATABASE_PORT")
        .unwrap_or_else(|_| {
            let port = 5432;

            log::warn!("No DATABASE_PORT environment variable found, using default port {}", port);

            port.to_string()
        })
        .parse()
        .expect("Failed to parse DATABASE_PORT environment variable")
});

pub static DATABASE_USER: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_USER")
        .unwrap_or_else(|_| {
            let user = "yuny_db_user".to_string();

            log::warn!("No DATABASE_USER environment variable found, using default user {}", user);

            user
        })
});

pub static DATABASE_NAME: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_NAME")
        .unwrap_or_else(|_| {
            let name = "yuny_db".to_string();

            log::warn!("No DATABASE_NAME environment variable found, using default database {}", name);

            name
        })
});

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

pub static JWT_SECRET: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_|
        {
            log::warn!("No JWT_SECRET environment variable found, using default secret");

            "9K+7wGkwH2In5yMFVWra4f8cBsuEBzIA3ZMo0SoRM09r8DeGuNjLyKWiHvEnSG4illto6RZvOjMrq+Nx40I/msKhiK/J0U499xjF2JK/1RI=".to_string()
        });

    // Make sure secret is a valid length
    if secret.len() < 64 {
        log::error!("JWT_SECRET must be at least 64 bytes long");
        std::process::exit(1);
    }

    secret.as_bytes().to_vec()
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