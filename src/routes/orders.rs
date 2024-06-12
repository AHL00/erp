use rocket::{http::Status, response::status::Custom};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json, Row};

use crate::{
    db::{FromDB, DB},
    routes::{auth::{AuthGuard, UserRow}, ListRequest},
    types::permissions::UserPermissionEnum,
};

use super::{auth::User, customers::Customer, inventory::InventoryItem, ApiError, ApiReturn};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct OrderMeta {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    /// This will be false if the order is retail
    pub customer: Customer,
    pub created_by_user: User,
    pub amount_paid: sqlx::types::BigDecimal,
    pub retail: bool,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct OrderItem {
    pub id: i32,
    pub inventory_item: InventoryItem,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) struct Order {
    #[serde(flatten)]
    pub meta: OrderMeta,
    pub items: Vec<OrderItem>,
}

/// GET /orders/<id>
/// Response: OrderMeta
#[rocket::get("/orders/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<rocket::serde::json::Json<OrderMeta>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT get_order($1)
        "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    let json = row
        .try_get_raw(0)?
        .as_str()
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    let order_meta = serde_json::from_str::<OrderMeta>(json)
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    // Re-encoding json may not be efficient but we may do some serde flattening later.
    // Also allows some error handling to be done in the future.
    // It also makes sure that the json layout is correct before sending it to the client.
    // If not perfomant enough, we can always just send the json variable directly.
    Ok(rocket::serde::json::Json(order_meta))
}

#[rocket::get("/orders/<id>/items")]
pub(super) async fn get_items(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<OrderItem>>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT get_order_items($1)
        "#,
    )
    .bind(id)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    let mut order_items = Vec::with_capacity(rows.len());

    for row in rows {
        let json = row
            .try_get_raw(0)?
            .as_str()
            .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

        let order_item = serde_json::from_str::<OrderItem>(json)
            .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

        order_items.push(order_item);
    }

    Ok(rocket::serde::json::Json(order_items))
}

#[rocket::get("/orders/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<rocket::serde::json::Json<i64>, ApiError> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT count(*) FROM orders
        "#,
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    Ok(rocket::serde::json::Json(count.0))
}

/// POST /orders/list
/// /// Request:
/// ```json
/// {
///     "range": {
///         "start": 0,
///         "end": 10
///     },
///     "sort": {
///         "column": "date_time",
///         "order": "asc"
///     }
///     "filter": [{
///         "column": "date_time",
///         "value": "...",
///     }]
/// }
/// ```
/// Response: Vec<Order>
///
/// Special columns: customer, created_by_user
#[rocket::post("/orders/list", data = "<req>")]
pub(super) async fn list(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
    req: rocket::serde::json::Json<ListRequest>,
) -> Result<rocket::serde::json::Json<Vec<OrderMeta>>, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let sorts_string = super::generate_sorts_string(&req.sorts);

    let (filters_string, filter_binds) =
        super::generate_filters_string(&req.filters, &mut current_param);

    let query_str = format!(
        r#"
        SELECT 
            orders.id,
            orders.date_time,
            orders.amount_paid,
            orders.retail,
            orders.notes,
            get_customer_json(orders.customer_id) AS customer,
            get_user_json(orders.created_by_user_id) AS created_by_user
        FROM orders
            INNER JOIN customers ON orders.customer_id = customers.id
            INNER JOIN users ON orders.created_by_user_id = users.id
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

    #[derive(FromRow, Debug)]
    struct OrderRow {
        id: i32,
        date_time: chrono::DateTime<chrono::Utc>,
        customer: sqlx_core::types::Json<Customer>,
        created_by_user: sqlx_core::types::Json<UserRow>,
        amount_paid: sqlx::types::BigDecimal,
        retail: bool,
        notes: String,
    }

    impl From<OrderRow> for OrderMeta {
        fn from(row: OrderRow) -> Self {
            Self {
                id: row.id,
                date_time: row.date_time,
                customer: row.customer.0,
                created_by_user: row.created_by_user.0.into(),
                amount_paid: row.amount_paid,
                retail: row.retail,
                notes: row.notes,
            }
        }
    }

    let query = sqlx::query_as(&query_str);

    let query = filter_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query_as(query));

    let query = query.bind(req.range.count).bind(req.range.offset);

    let orders: Vec<OrderMeta> = query
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError(Status::NotFound, "No orders found".to_string()),
            e => ApiError(Status::InternalServerError, e.to_string()),
        })
        .map(|rows: Vec<OrderRow>| rows.into_iter().map(OrderMeta::from).collect())?;

    Ok(rocket::serde::json::Json(orders))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderItemPostRequest {
    pub inventory_item_id: i32,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderPostRequest {
    pub customer_id: i32,
    pub retail: bool,
    pub notes: String,
    pub amount_paid: sqlx::types::BigDecimal,
    pub items: Vec<OrderItemPostRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub struct OrderItemPatchRequest {
    pub inventory_item_id: Option<i32>,
    pub quantity: Option<i32>,
    pub price: Option<sqlx::types::BigDecimal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderPatchRequest {
    pub customer_id: Option<i32>,
    pub retail: Option<bool>,
    pub notes: Option<String>,
    pub amount_paid: Option<sqlx::types::BigDecimal>,
    pub items: Vec<OrderItemPatchRequest>,
}

#[rocket::post("/orders", data = "<req>")]
pub(super) async fn post(
    req: rocket::serde::json::Json<OrderPostRequest>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let req = req.into_inner();

    let user_id = auth.auth_info.user.id;

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO orders (customer_id, created_by_user_id, amount_paid, retail, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(req.customer_id)
    .bind(user_id)
    .bind(req.amount_paid)
    .bind(req.retail)
    .bind(req.notes)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

#[rocket::delete("/orders/<id>")]
pub(super) async fn delete(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<Status, ApiError> {
    sqlx::query(
        r#"
        DELETE FROM orders
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&mut **db)
    .await?;

    Ok(Status::NoContent)
}
