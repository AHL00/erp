use crate::routes::auth::AuthGuard;
use crate::routes::search::SearchRequest;
use crate::routes::{ListRequest, SqlType};
use crate::{db::DB, types::permissions::UserPermissionEnum};
use bigdecimal::BigDecimal;
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::{ApiError, ApiReturn};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct InventoryItem {
    pub id: i32,
    pub name: String,
    /// A decimal number with a precision of 2 decimal places
    pub price: BigDecimal,
    pub stock: i32,
    pub quantity_per_box: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct InventoryItemPatchRequest {
    pub name: Option<String>,
    /// A decimal number with a precision of 2 decimal places
    pub price: Option<BigDecimal>,
    pub stock: Option<i32>,
    pub quantity_per_box: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct InventoryItemPostRequest {
    pub name: String,
    /// A decimal number with a precision of 2 decimal places
    pub price: BigDecimal,
    pub stock: i32,
    pub quantity_per_box: i32,
}

// GET /inventory/count
// Response: i64
#[rocket::get("/inventory/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_READ as u32 }>,
) -> Result<Json<i64>, ApiError> {
    // WHERE deleted = false
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM inventory")
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(count))
}

/// POST /inventory/list
/// Request:
/// ```json
/// {
///     "range": {
///         "start": 0,
///         "end": 10
///     },
///     "sort": {
///         "column": "name",
///         "order": "asc"
///     }
///     "filter": [{
///         "column": "name",
///         "value": "a"
///     }]
/// }
/// ```
/// Response: Vec<InventoryItem>
#[rocket::post("/inventory/list", data = "<req>")]
pub(super) async fn list(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_READ as u32 }>,
    req: Json<ListRequest>,
) -> Result<Json<Vec<InventoryItem>>, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let sorts_string = super::generate_sorts_string(&req.sorts);

    let (filters_string, filter_binds) =
        super::generate_filters_string(&req.filters, &mut current_param);

    let query_str = format!(
        r#"
        SELECT * FROM inventory
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

    let query = query.bind(req.range.count).bind(req.range.offset);

    let data: Vec<InventoryItem> = query.fetch_all(&mut **db).await.map_err(|e| match e {
        sqlx::Error::ColumnNotFound(column) => {
            ApiError(Status::BadRequest, format!("Column not found: {}", column))
        }
        _ => e.into(),
    })?;

    Ok(Json(data))
}

#[rocket::get("/inventory/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_READ as u32 }>,
) -> Result<Json<InventoryItem>, ApiError> {
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

/// POST /inventory
/// Request: InventoryItem
/// Note: id is ignored
/// Response: id or Status
#[rocket::post("/inventory", data = "<item>")]
pub(super) async fn post(
    item: Json<InventoryItemPostRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_WRITE as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let item = item.into_inner();

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO inventory (name, price, stock, quantity_per_box)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&item.name)
    .bind(&item.price)
    .bind(item.stock)
    .bind(item.quantity_per_box)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

/// PATCH /inventory
/// Request: InventoryItem
/// Response: ApiError or Status
#[rocket::patch("/inventory/<id>", data = "<item>")]
pub(super) async fn patch(
    item: Json<InventoryItemPatchRequest>,
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::INVENTORY_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    let req = item.into_inner();

    let mut current_param_index = 1;

    let columns = vec![
        req.name.as_ref().map(|_| "name"),
        req.price.as_ref().map(|_| "price"),
        req.stock.as_ref().map(|_| "stock"),
        req.quantity_per_box.as_ref().map(|_| "quantity_per_box"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets_string = super::generate_sets_string(&columns, &mut current_param_index);

    if sets_string.is_empty() {
        return Ok(Status::NoContent);
    }

    let set_binds = vec![
        req.name.as_ref().map(|v| SqlType::String(v.clone())),
        req.price.as_ref().map(|v| SqlType::BigDecimal(v.clone())),
        req.stock.as_ref().map(|v| SqlType::Int(v.clone())),
        req.quantity_per_box
            .as_ref()
            .map(|v| SqlType::Int(v.clone())),
    ]
    .into_iter()
    .flatten();

    let query_str = format!(
        r#"
        UPDATE inventory
        SET {}
        WHERE id = ${}
        RETURNING id
        "#,
        sets_string, current_param_index
    );

    let query = sqlx::query(&query_str);

    let query = set_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query(query));

    query
        .bind(id)
        .fetch_one(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::BadRequest, format!("Row with id {} not found", id))
            }
            _ => e.into(),
        })?;

    Ok(Status::NoContent)
}

#[rocket::post("/inventory/search", data = "<req>")]
pub(super) async fn search(
    req: Json<SearchRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::CUSTOMERS_READ as u32 }>,
) -> Result<Json<Vec<InventoryItem>>, ApiError> {
    let req = req.into_inner();

    let column_ts_name = format!("{}_ts", req.column);

    let query_str = format!(
        r#"
    SELECT *
    FROM inventory
    WHERE {} @@ phraseto_tsquery('english', $1)
    LIMIT $2
    "#,
        column_ts_name
    );

    let query = sqlx::query_as(&query_str);

    let data: Vec<InventoryItem> = query
        .bind(&req.search)
        .bind(req.count)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::ColumnNotFound(column) => ApiError(
                Status::BadRequest,
                format!(
                    "Column not found or doesn't have a text search index: {}",
                    column
                ),
            ),
            _ => e.into(),
        })?;

    Ok(Json(data))
}
