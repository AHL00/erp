use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
    db::DB, routes::auth::AuthGuard, types::permissions::UserPermissionEnum,
};

use super::{ApiError, ApiReturn};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Customer {
    id: i32,
    name: String,
    phone: String,
    address: String,
    notes: String,
}

// #[rocket::get("/customers/count")]
// pub(super) async fn count(mut db: crate::db::DB,
// _auth: crate::routes::auth::AuthGuard<{ crate::types::permissions::UserPermissionEnum::CUSTOMERS_READ as u32 }>,
// ) -> Result<rocket::serde::json::Json<i32>, rocket::http::Status> {
//     let count = sqlx::query_scalar("SELECT COUNT(*) FROM customers")
//         .fetch_one(&mut **db)
//         .await
//         .map_err(|_| rocket::http::Status::InternalServerError)?;

//     Ok(rocket::serde::json::Json(count))
// }

#[rocket::get("/customers/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::CUSTOMERS_READ as u32 }>,
) -> Result<Json<i32>, Status> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM customers")
        .fetch_one(&mut **db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(count))
}

#[rocket::get("/customers/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::CUSTOMERS_READ as u32 }>,
) -> Result<Json<Customer>, ApiError> {
    let item = sqlx::query_as(
        r#"
        SELECT * FROM inventory
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            ApiError(Status::BadRequest, format!("Row with id {} not found", id))
        }
        _ => e.into(),
    })?;

    Ok(Json(item))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct CustomerPostRequest {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub notes: String,
}

#[rocket::post("/customers", data = "<item>")]
pub(super) async fn post(
    item: Json<CustomerPostRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_WRITE as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let item = item.into_inner();

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO customers (name, phone, address, notes)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&item.name)
    .bind(&item.phone)
    .bind(&item.address)
    .bind(&item.notes)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct CustomerPutRequest {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub notes: String,
}

#[rocket::put("/customers/<id>", data = "<item>")]
pub(super) async fn put(
    item: Json<CustomerPutRequest>,
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    let item = item.into_inner();

    sqlx::query(
        r#"
        UPDATE customers
        SET name = $1, phone = $2, address = $3, notes = $4
        WHERE id = $5
        "#,
    )
    .bind(&item.name)
    .bind(&item.phone)
    .bind(&item.address)
    .bind(&item.notes)
    .bind(id)
    .execute(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            ApiError(Status::BadRequest, format!("Row with id {} not found", id))
        }
        _ => e.into(),
    })?;

    Ok(Status::Ok)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct CustomerPatchRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

#[rocket::patch("/customers/<id>", data = "<item>")]
pub(super) async fn patch(
    item: Json<CustomerPatchRequest>,
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    let item = item.into_inner();

    let mut current_param = 1;

    let columns = &["name", "phone", "address", "notes"];

    let sets_string = super::generate_sets_string(columns, &mut current_param);

    if sets_string.is_empty() {
        return Ok(Status::Ok);
    }

    let query_str = format!(
        r#"
        UPDATE customers
        SET {}
        WHERE id = ${}
        "#,
        sets_string, current_param
    );

    let data = sqlx::query(&query_str)
        .bind(&item.name)
        .bind(&item.phone)
        .bind(&item.address)
        .bind(&item.notes)
        .bind(id)
        .execute(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::BadRequest, format!("Row with id {} not found", id))
            }
            _ => e.into(),
        })?;

    Ok(Status::Ok)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct CustomerListRequest {
    pub range: super::ListRange,
    pub sorts: Vec<super::ListSort>,
    pub filters: Vec<super::ListFilter>,
}