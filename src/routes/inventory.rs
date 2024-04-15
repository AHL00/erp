use crate::db::DB;
use bigdecimal::BigDecimal;
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::{ApiError, ApiReturn, FilterOperator, SortOrder, SqlType};

// GET /inventory/count
// Response: i32
#[rocket::get("/inventory/count")]
pub(super) async fn count(mut db: DB) -> Result<Json<i32>, Status> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM inventory")
        .fetch_one(&mut **db)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(count))
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct InventoryItemPatchRequest {
    pub name: Option<String>,
    /// A decimal number with a precision of 2 decimal places
    pub price: Option<BigDecimal>,
    pub stock: Option<i32>,
    pub quantity_per_box: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct InventoryItemPutRequest {
    pub name: String,
    /// A decimal number with a precision of 2 decimal places
    pub price: BigDecimal,
    pub stock: i32,
    pub quantity_per_box: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct InventoryItemPostRequest {
    pub name: String,
    /// A decimal number with a precision of 2 decimal places
    pub price: BigDecimal,
    pub stock: i32,
    pub quantity_per_box: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct InventoryItemListSort {
    column: String,
    order: SortOrder,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct InventoryItemListFilter {
    column: String,
    operator: FilterOperator,
    value: SqlType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct InventoryItemListRange {
    /// Number of items to send
    count: i32,
    /// First item's index
    offset: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct InventoryItemListRequest {
    #[ts(inline)]
    range: InventoryItemListRange,
    #[ts(inline)]
    sorts: Vec<InventoryItemListSort>,
    #[ts(inline)]
    filters: Vec<InventoryItemListFilter>,
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
    req: Json<InventoryItemListRequest>,
) -> Result<Json<Vec<InventoryItem>>, (Status, String)> {
    let req = req.into_inner();

    let mut sql_arg_index = 1;

    let mut filter_binds = vec![];

    let filter_string = req
        .filters
        .iter()
        .enumerate()
        .map(|(i, filter)| {
            let str = format!(
                "{} {} {} ${}",
                if i == 0 { "WHERE" } else { "AND" },
                filter.column,
                filter.operator.to_sql(),
                sql_arg_index
            );

            filter_binds.push(filter.value.clone());

            sql_arg_index += 1;

            str
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut sort_string = req
        .sorts
        .iter()
        .enumerate()
        .map(|(i, sort)| {
            format!(
                "{} {} {}",
                if i == 0 { "ORDER BY" } else { "," },
                sort.column,
                match sort.order {
                    SortOrder::Asc => "ASC",
                    SortOrder::Desc => "DESC",
                }
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    // If ends in a comma, remove it
    sort_string = sort_string.trim_end_matches(',').to_string();

    let query_string = format!(
        "SELECT * FROM inventory {} {} LIMIT ${} OFFSET ${}",
        filter_string,
        sort_string,
        sql_arg_index,
        sql_arg_index + 1
    );

    let query = sqlx::query_as(&query_string);

    let query = filter_binds
        .iter()
        .fold(query, |query, value| value.bind_to_query_as(query));

    // Bind the last two range values
    let query = query.bind(req.range.count).bind(req.range.offset);

    log::info!("Query: {}", query_string);

    let data: Vec<InventoryItem> = query.fetch_all(&mut **db).await.map_err(|e| {
        match e {
            sqlx::Error::ColumnNotFound(column) => {
                (Status::BadRequest, format!("Column not found: {}", column))
            }
            // TODO: Make sure this error won't contain sensitive information
            _ => (Status::InternalServerError, e.to_string()),
        }
    })?;

    Ok(Json(data))
}

/// GET /inventory/<id>
/// Response: InventoryItem
#[rocket::get("/inventory/<id>")]
pub(super) async fn get(id: i32, mut db: DB) -> Result<Json<InventoryItem>, ApiError> {
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
            ApiError(Status::BadRequest, format!("Item with id {} not found", id))
        }
        _ => e.into(),
    })?;

    Ok(Json(item))
}

/// POST /inventory
/// Request: InventoryItem
/// Response: id or Status
#[rocket::post("/inventory", data = "<item>")]
pub(super) async fn post(item: Json<InventoryItemPutRequest>, mut db: DB) -> Result<ApiReturn<i32>, ApiError> {
    let item = item.into_inner();

    let id: (i32, ) = sqlx::query_as(
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

/// PUT /inventory/<id>
/// Request: InventoryItem
/// Response: ApiError or Status
#[rocket::put("/inventory/<id>", data = "<item>")]
pub(super) async fn put(
    item: Json<InventoryItemPutRequest>,
    id: i32,
    mut db: DB,
) -> Result<Status, ApiError> {
    let item = item.into_inner();

    sqlx::query(
        r#"
        UPDATE inventory
        SET name = $1, price = $2, stock = $3, quantity_per_box = $4
        WHERE id = $5
        RETURNING id
        "#,
    )
    .bind(&item.name)
    .bind(&item.price)
    .bind(item.stock)
    .bind(item.quantity_per_box)
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            ApiError(Status::BadRequest, format!("Item with id {} not found", id))
        }
        _ => e.into(),
    })?;

    Ok(Status::Ok)
}

/// PATCH /inventory
/// Request: InventoryItem
/// Response: ApiError or Status
// TODO: Maybe take in an array of items? What would the use case be?
#[rocket::patch("/inventory/<id>", data = "<item>")]
pub(super) async fn patch(
    item: Json<InventoryItemPatchRequest>,
    id: i32,
    mut db: DB,
) -> Result<Status, ApiError> {
    let item = item.into_inner();

    let sets_string = vec![
        item.name.as_ref().map(|_| "name = $1"),
        item.price.as_ref().map(|_| "price = $2"),
        item.stock.as_ref().map(|_| "stock = $3"),
        item.quantity_per_box
            .as_ref()
            .map(|_| "quantity_per_box = $4"),
    ];

    let sets_string = sets_string
        .into_iter()
        .filter_map(|x| x)
        .collect::<Vec<_>>()
        .join(", ");

    if sets_string.is_empty() {
        return Err(ApiError(
            Status::BadRequest,
            "No fields to update".to_string(),
        ));
    }

    sqlx::query(
        format!(
            "
        UPDATE inventory \
        SET {} \
        WHERE id = $5 \
        RETURNING id \
        ",
            sets_string.as_str()
        )
        .as_str(),
    )
    .bind(&item.name)
    .bind(&item.price)
    .bind(item.stock)
    .bind(item.quantity_per_box)
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            ApiError(Status::BadRequest, format!("Item with id {} not found", id))
        }
        _ => e.into(),
    })?;

    Ok(Status::Ok)
}
