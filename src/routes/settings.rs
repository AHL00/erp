use rocket::{http::Status, serde::json::Json};

use crate::{
    db::DB,
    settings::{
        ensure_settings_exist, get_setting, get_settings, set_setting, Setting, SettingRow,
    },
    types::permissions::UserPermissionEnum,
};

use super::{auth::AuthGuard, ApiError};

#[rocket::get("/settings/get_all")]
pub(super) async fn get_all(mut db: DB) -> Result<Json<Vec<Setting>>, ApiError> {
    let settings = sqlx::query_as::<_, SettingRow>(
        r#"
        SELECT key, long_name, description, value
        FROM settings
        ORDER BY key
        "#,
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to fetch settings: {}", e),
        )
    })?;

    if settings.is_empty() {
        ensure_settings_exist(&mut db).await.map_err(|e| {
            ApiError(
                Status::InternalServerError,
                format!("Failed to ensure settings exist: {}", e),
            )
        })?;

        log::info!("Settings table was empty, so it was populated with default values");
    }

    let settings = sqlx::query_as::<_, SettingRow>(
        r#"
        SELECT key, long_name, description, value
        FROM settings
        ORDER BY key
        "#,
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to fetch settings: {}", e),
        )
    })?;

    Ok(Json(settings.into_iter().map(Setting::from).collect()))
}

#[rocket::get("/settings/get_one/<key>")]
pub(super) async fn get(key: &str, mut db: DB) -> Result<Json<Setting>, ApiError> {
    let setting = get_setting(&mut db, key).await?;

    if let Some(setting) = setting {
        Ok(Json(setting))
    } else {
        Err(ApiError(Status::NotFound, "Setting not found".to_string()))
    }
}

#[derive(serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub(super) struct MultipleSettingsRequest {
    keys: Vec<String>,
}

#[rocket::post("/settings/get_multiple", data = "<settings>")]
pub(super) async fn get_multiple(
    settings: Json<MultipleSettingsRequest>,
    mut db: DB,
) -> Result<Json<Vec<Setting>>, ApiError> {
    let keys = settings.0.keys.into_iter().collect::<Vec<_>>();

    let settings = get_settings(&mut db, keys).await?;

    Ok(Json(settings))
}

#[rocket::post("/settings/set", data = "<setting>")]
pub(super) async fn set(
    setting: Json<Setting>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SETTINGS as u32 }>,
) -> Result<(), ApiError> {
    set_setting(&mut db, setting.0.into()).await?;

    Ok(())
}
