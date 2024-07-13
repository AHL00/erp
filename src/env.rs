pub fn public_dir() -> String {
    std::env::var("PUBLIC_DIR")
        .unwrap_or_else(|_| {
            log::warn!("No PUBLIC_DIR environment variable found, using default public directory \"\"");

            "".to_string()
        })
}

pub fn api_root() -> String {
    std::env::var("API_ROOT")
        .unwrap_or_else(|_| {
            log::warn!("No API_ROOT environment variable found, using default API root /");

            "/".to_string()
        })
}