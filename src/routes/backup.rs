use rocket::{futures::StreamExt, http::Status, outcome};
use sqlx::Acquire;

use crate::{
    db::{self, DB},
    types::permissions::UserPermissionEnum,
};

use super::{auth::AuthGuard, ApiError};

#[rocket::get("/db/backup")]
pub async fn backup(
    _auth: AuthGuard<{ UserPermissionEnum::MANAGE_DB as u32 }>,
) -> Result<String, ApiError> {
    let db_url = std::env::var("ROCKET_DATABASES");

    let db_url = match db_url {
        Ok(url) => url,
        Err(e) => {
            log::error!("Error getting database URL: {}", e);
            return Err(ApiError(
                rocket::http::Status::InternalServerError,
                "Database URL not found".to_string(),
            ));
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
        .arg("--inserts")
        .output()
        .map_err(|e| ApiError(rocket::http::Status::InternalServerError, e.to_string()))?;

    if !output.status.success() {
        return Err(ApiError(
            rocket::http::Status::InternalServerError,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let out_string = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(out_string)
}

// #[rocket::post("/db/restore", data = "<sql_data>")]
// pub async fn restore(
//     _auth: AuthGuard<{ UserPermissionEnum::MANAGE_DB as u32 }>,
//     sql_data: rocket::Data<'_>,
//     mut db: DB,
// ) -> Result<Status, ApiError> {
//     let mut sql_data = {
//         sql_data
//             .open(rocket::data::ByteUnit::max_value())
//             .into_string()
//             .await
//             .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?
//             .to_string()
//     };

    
//     // TODO: Track down weird bug where the string is getting 
//     // quotation marks added to the ends of it.
//     // For now, just remove them.
//     if sql_data.starts_with('"') {
//         sql_data = sql_data[1..].to_string();
//     }
    
//     if sql_data.ends_with('"') {
//         sql_data.pop();
//     }
    
//     // sql_data = sql_data.replace("\\n", "");
//     // sql_data = sql_data.replace("\n", "");
//     // 
//     log::info!("{}", sql_data);

//     let mut transaction = db.begin().await.map_err(|e| {
//         ApiError(
//             Status::InternalServerError,
//             format!("Failed to start transaction: {}", e),
//         )
//     })?;

//     sqlx::query(
//         r#"
//         DROP SCHEMA public CASCADE;
//         "#,
//     )
//     .execute(&mut *transaction)
//     .await
//     .map_err(|e| {
//         ApiError(
//             rocket::http::Status::InternalServerError,
//             format!("Failed to drop schema: {}", e),
//         )
//     })?;

//     use sqlx::Executor;

//     let mut res = transaction.execute(
//         sqlx::raw_sql(&sql_data)
//     ).await;

//     if let Err(e) = res {
//         transaction.rollback().await.map_err(|e| {
//             ApiError(
//                 Status::InternalServerError,
//                 format!("Failed to rollback transaction: {}", e),
//             )
//         })?;

//         return Err(ApiError(
//             rocket::http::Status::InternalServerError,
//             format!("Failed to execute restore SQL: {}", e),
//         ));
//     }

//     let rows_affected = res.unwrap().rows_affected();

//     log::info!("Rows affected: {}", rows_affected);

//     // Check whether schema was created
//     // Run schema.sql, if nothing was affected, then its all good
//     // let schema_check = sqlx::query(
//     //     r#"
//     //     SELECT EXISTS (
//     //         SELECT 1
//     //         FROM   information_schema.schemata 
//     //         WHERE  schema_name = 'public'
//     //     );
//     //     "#,
//     // )
//     // .fetch_one(&mut *transaction)
//     // .await
//     // .map_err(|e| {
//     //     ApiError(
//     //         rocket::http::Status::BadRequest,
//     //         format!("Failed to check for schema: {}", e),
//     //     )
//     // })?;

//     // use sqlx::Row;
//     // if schema_check.get::<bool, _>("exists") == false {
//     //     return Err(ApiError(
//     //         rocket::http::Status::BadRequest,
//     //         "Backup file does not contain schema".to_string(),
//     //     ));
//     // }

//     transaction.commit().await.map_err(|e| {
//         ApiError(
//             Status::InternalServerError,
//             format!("Failed to commit transaction: {}", e),
//         )
//     })?;

//     Ok(Status::Ok)
// }
