use crate::db::DB;
use bigdecimal::BigDecimal;
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query};

use super::{FilterOperator, SortOrder, SqlType};

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
