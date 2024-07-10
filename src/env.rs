pub fn public_dir() -> String {
    std::env::var("PUBLIC_DIR")
        .unwrap_or_else(|_| {
            log::warn!("No PUBLIC_DIR environment variable found, using default public directory \"\"");

            "".to_string()
        })
}

pub fn host() -> String {
    std::env::var("HOST")
        .unwrap_or_else(|_| {
            log::warn!("No HOST environment variable found, using default host localhost");

            "127.0.0.1".to_string()
        })
}

pub fn port() -> u16 {
    std::env::var("PORT")
        .unwrap_or_else(|_| {
            log::warn!("No PORT environment variable found, using default port 8080");

            "8080".to_string()
        })
        .parse()
        .expect("Failed to parse PORT environment variable")
}

pub fn api_root() -> String {
    std::env::var("API_ROOT")
        .unwrap_or_else(|_| {
            log::warn!("No API_ROOT environment variable found, using default API root /");

            "/".to_string()
        })
}