use rocket_download_response::DownloadResponse;

use crate::{db, types::permissions::UserPermissionEnum};

use super::{auth::AuthGuard, ApiError};

#[rocket::get("/backup")]
pub async fn backup(
    // _auth: AuthGuard<{ UserPermissionEnum::MANAGE_DB as u32 }>,
) -> Result<String, ApiError> {
    let db_url = std::env::var("ROCKET_DATABASES");

    let db_url = match db_url {
        Ok(url) => url,
        Err(e) => {
            log::error!("Error getting database URL: {}", e);
            return Err(ApiError(
                rocket::http::Status::InternalServerError,
                "Database URL not found".to_string(),
            ))
        }
    };

    let db_url = {
        let start = db_url.find("postgresql://").ok_or(ApiError(
            rocket::http::Status::InternalServerError,
            "Database URL not found".to_string(),
        ))?;

        // End at next " or }
        let end = db_url[start..]
            .find('"')
            .or(db_url[start..].find('}'))
            .ok_or(ApiError(
                rocket::http::Status::InternalServerError,
                "Database URL not found".to_string(),
            ))?;

        &db_url[start..start + end]
    };

    log::info!("Backing up database: {:16}...", db_url);

    let output = std::process::Command::new("pg_dump")
        .arg(db_url)
        .arg("-n")
        .arg("public")
        .output()
        .map_err(|e| ApiError(rocket::http::Status::InternalServerError, e.to_string()))?;

    if !output.status.success() {
        return Err(ApiError(
            rocket::http::Status::InternalServerError,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
