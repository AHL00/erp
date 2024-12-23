use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
    db::{FromDB, DB},
    routes::{auth::AuthGuard, search::SearchRequest, ListRequest},
    types::permissions::UserPermissionEnum,
};

use super::{ApiError, ApiReturn, SqlType};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Supplier {
    id: i32,
    name: String,
    phone: String,
    address: String,
    notes: String,
}

impl FromDB for Supplier {
    async fn from_db(id: i32, db: &mut crate::db::DB) -> Result<Self, ApiError> {
        sqlx::query_as(
            r#"
                SELECT * FROM suppliers
                WHERE id = $1
                "#,
        )
        .bind(id)
        .fetch_one(&mut ***db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::BadRequest, format!("Row with id {} not found", id))
            }
            _ => e.into(),
        })
    }
}

#[rocket::get("/suppliers/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_READ as u32 }>,
) -> Result<Json<i64>, ApiError> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM suppliers")
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(count))
}

#[rocket::post("/suppliers/list", data = "<req>")]
pub(super) async fn list(
    req: Json<ListRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_READ as u32 }>,
) -> Result<Json<Vec<Supplier>>, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let sorts_string = super::generate_sorts_string(&req.sorts);

    let (filters_string, filter_binds) =
        super::generate_filters_string(&req.filters, &mut current_param);

    let query_str = format!(
        r#"
        SELECT * FROM suppliers
        {}
        {}
        LIMIT ${}
        OFFSET ${}
        "#,
        filters_string,
        sorts_string,
        current_param,
        current_param + 1
    );

    let query = sqlx::query_as(&query_str);

    let query = filter_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query_as(query));

    let data = query
        .bind(req.range.count)
        .bind(req.range.offset)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::ColumnNotFound(column) => {
                ApiError(Status::BadRequest, format!("Column not found: {}", column))
            }
            _ => e.into(),
        })?;

    Ok(Json(data))
}

#[rocket::get("/suppliers/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_READ as u32 }>,
) -> Result<Json<Supplier>, ApiError> {
    Ok(Json(Supplier::from_db(id, &mut db).await?))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct SupplierPostRequest {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub notes: String,
}

#[rocket::post("/suppliers", data = "<item>")]
pub(super) async fn post(
    item: Json<SupplierPostRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_CREATE as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let item = item.into_inner();

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO suppliers (name, phone, address, notes)
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
pub(super) struct SupplierPatchRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

#[rocket::patch("/suppliers/<id>", data = "<req>")]
pub(super) async fn patch(
    id: i32,
    req: Json<SupplierPatchRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_UPDATE as u32 }>,
) -> Result<Status, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let columns = vec![
        req.name.as_ref().map(|_| "name"),
        req.phone.as_ref().map(|_| "phone"),
        req.address.as_ref().map(|_| "address"),
        req.notes.as_ref().map(|_| "notes"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets_string = super::generate_sets_string(&columns, &mut current_param);

    if sets_string.is_empty() {
        return Ok(Status::Ok);
    }

    let set_binds = vec![
        req.name.as_ref().map(|v| SqlType::String(v.clone())),
        req.phone.as_ref().map(|v| SqlType::String(v.clone())),
        req.address.as_ref().map(|v| SqlType::String(v.clone())),
        req.notes.as_ref().map(|v| SqlType::String(v.clone())),
    ]
    .into_iter()
    .flatten();

    let query_str = format!(
        r#"
        UPDATE suppliers
        SET {}
        WHERE id = ${}
        "#,
        sets_string, current_param
    );

    let query = sqlx::query(&query_str);

    let query = set_binds.fold(query, |query, value| value.bind_to_query(query));

    query
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

#[rocket::post("/suppliers/search", data = "<req>")]
pub(super) async fn search(
    req: Json<SearchRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::SUPPLIERS_READ as u32 }>,
) -> Result<Json<Vec<Supplier>>, ApiError> {
    let req = req.into_inner();

    let x = req
        .column
        .unwrap_or(req.nested_access.unwrap_or("id".to_string()));

    let query_str = format!(
        r#"
        SELECT *, word_similarity($1, {}::text) AS sml
        FROM suppliers
        WHERE $1 <% {}::text
        ORDER BY sml DESC, {}::text
        LIMIT $2
        "#,
        x, x, x
    );

    let query = sqlx::query_as(&query_str);

    let data = query
        .bind(req.search)
        .bind(req.count)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::ColumnNotFound(column) => {
                ApiError(Status::BadRequest, format!("Column not found: {}", column))
            }
            _ => e.into(),
        })?;

    Ok(Json(data))
}
