use crate::{
    db::DB,
    routes::{
        inventory::count_impl,
        search::SearchRequest,
        ApiError, ListRequest,
    },
};
use bigdecimal::BigDecimal;
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// GET /inventory/count
/// Response: i64
#[rocket::get("/inventory/count")]
pub(super) async fn count(db: DB) -> Result<Json<i64>, ApiError> {
    count_impl(db).await
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct PublicInventoryItem {
    pub id: i32,
    pub name: String,
    /// A decimal number with a precision of 2 decimal places
    pub price: BigDecimal,
    pub quantity_per_box: i32,
}

#[rocket::post("/inventory/list", data = "<req>")]
pub(super) async fn list(
    mut db: DB,
    req: Json<ListRequest>,
) -> Result<Json<Vec<PublicInventoryItem>>, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let sorts_string = super::generate_sorts_string(&req.sorts);

    let (filters_string, filter_binds) =
        super::generate_filters_string(&req.filters, &mut current_param);

    // TODO: Can they sort by stock even if it's not queried?
    let query_str = format!(
        r#"
        SELECT id, name, price, quantity_per_box FROM inventory
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

    let data: Vec<PublicInventoryItem> = query.fetch_all(&mut **db).await.map_err(|e| match e {
        sqlx::Error::ColumnNotFound(column) => {
            ApiError(Status::BadRequest, format!("Column not found: {}", column))
        }
        _ => e.into(),
    })?;

    Ok(Json(data))
}

#[rocket::get("/inventory/<id>")]
pub(super) async fn get(id: i32, mut db: DB) -> Result<Json<PublicInventoryItem>, ApiError> {
    let item = sqlx::query_as(
        r#"
        SELECT id, name, price, quantity_per_box FROM inventory
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

#[rocket::post("/inventory/search", data = "<req>")]
pub(super) async fn search(
    req: Json<SearchRequest>,
    mut db: DB,
) -> Result<Json<Vec<PublicInventoryItem>>, ApiError> {
    let req = req.into_inner();

    let query_str = format!(
        r#"
        SELECT *, word_similarity($1, "{}") AS sml
        FROM inventory
        WHERE $1 <% "{}"
        ORDER BY sml DESC, "{}"
        LIMIT $2
        "#,
        req.column, req.column, req.column
    );

    let query = sqlx::query_as(&query_str);

    let data: Vec<PublicInventoryItem> = query
        .bind(&req.search)
        .bind(req.count)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::ColumnNotFound(column) => ApiError(
                Status::BadRequest,
                format!(
                    "Column not found: {}",
                    column
                ),
            ),
            _ => e.into(),
        })?;

    Ok(Json(data))
}
