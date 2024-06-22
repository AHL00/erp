use rocket::{http::Status, response::status::Custom, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Row, Acquire};

use crate::{
    db::DB,
    routes::{
        auth::{AuthGuard, UserRow},
        ListRequest,
    },
    types::permissions::UserPermissionEnum,
};

use super::{
    auth::User, customers::Customer, inventory::InventoryItem, ApiError, ApiReturn, SqlType,
};

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

/// GET /orders/<id>
/// Response: OrderMeta
#[rocket::get("/orders/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<Json<OrderMeta>, ApiError> {
    let order_meta: OrderMeta = sqlx::query_as(
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
        WHERE orders.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
    .map(|row: OrderRow| row.into())?;

    Ok(order_meta.into())
}

#[rocket::get("/orders/<id>/items")]
pub(super) async fn get_items(
    id: i32,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<OrderItem>>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            *
        FROM order_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE order_id = $1
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
    auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
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
/// Nested columns: customer, created_by_user
#[rocket::post("/orders/list", data = "<req>")]
pub(super) async fn list(
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
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
}

#[rocket::post("/orders", data = "<req>")]
pub(super) async fn post(
    req: rocket::serde::json::Json<OrderPostRequest>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
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

#[rocket::patch("/orders/<id>", data = "<req>")]
pub(super) async fn patch(
    id: i32,
    req: rocket::serde::json::Json<OrderPatchRequest>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    let req = req.into_inner();

    let mut current_param_index = 1;

    let columns = vec![
        req.customer_id.as_ref().map(|_| "customer_id"),
        req.retail.as_ref().map(|_| "retail"),
        req.notes.as_ref().map(|_| "notes"),
        req.amount_paid.as_ref().map(|_| "amount_paid"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets_string = super::generate_sets_string(&columns, &mut current_param_index);

    if sets_string.is_empty() {
        return Ok(Status::NoContent);
    }

    log::info!("Request: {:?}", req);

    let set_binds = vec![
        req.customer_id.as_ref().map(|v| SqlType::Int(v.clone())),
        req.retail.as_ref().map(|v| SqlType::Boolean(v.clone())),
        req.notes.as_ref().map(|v| SqlType::String(v.clone())),
        req.amount_paid
            .as_ref()
            .map(|v| SqlType::BigDecimal(v.clone())),
    ]
    .into_iter()
    .flatten();

    let set_binds = set_binds.collect::<Vec<SqlType>>();

    log::info!("Set binds: {:?}", set_binds);

    let query_str = format!(
        r#"
        UPDATE orders
        SET {}
        WHERE id = ${}
        "#,
        sets_string, current_param_index
    );

    log::info!("Query: {}", query_str);

    let mut query = sqlx::query(&query_str);

    query = set_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query(query));

    query = query.bind(id);

    query.execute(&mut **db).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError(
            Status::BadRequest,
            format!("Order with id {} not found", id),
        ),
        _ => e.into(),
    })?;

    Ok(Status::NoContent)
}

/// Post one or more items to an order
/// POST /orders/<id>/items
#[rocket::post("/orders/<id>/items", data = "<req>")]
pub(super) async fn post_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<OrderItemPostRequest>>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
) -> Result<ApiReturn<Vec<i32>>, ApiError> {
    let requests = req.into_inner();

    let mut ids = vec![];

    let mut transaction = db.begin().await.map_err(|e| ApiError(
        Status::InternalServerError,
        format!("Failed to start transaction: {}", e),
    ))?;

    for (i, req) in requests.into_iter().enumerate() {
        let id: Result<(i32,), _> = sqlx::query_as(
            r#"
        INSERT INTO order_items (order_id, inventory_id, quantity, price)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        )
        .bind(id)
        .bind(req.inventory_item_id)
        .bind(req.quantity)
        .bind(req.price)
        .fetch_one(&mut *transaction)
        .await;

        if let Err(error) = id {
            transaction.rollback().await.map_err(|e| ApiError(
                Status::InternalServerError,
                format!("Failed to rollback transaction: {}", e),
            ))?;

            return Err(ApiError(
                Status::InternalServerError,
                format!("Failed to insert order item request at index {}: {}", i, error),
            ));
        }

        ids.push(id.unwrap().0);
    };

    transaction.commit().await.map_err(|e| ApiError(
        Status::InternalServerError,
        format!("Failed to commit transaction: {}", e),
    ))?;

    Ok(ApiReturn(Status::Created, ids))
}

#[rocket::patch("/orders/<id>/items", data = "<req>")]
pub(super) async fn patch_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<OrderItemPatchRequest>>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    let requests = req.into_inner();

    let mut transaction = db.begin().await.map_err(|e| ApiError(
        Status::InternalServerError,
        format!("Failed to start transaction: {}", e),
    ))?;

    for req in requests {
        let mut current_param_index = 1;

        let columns = vec![
            req.inventory_item_id.as_ref().map(|_| "inventory_id"),
            req.quantity.as_ref().map(|_| "quantity"),
            req.price.as_ref().map(|_| "price"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<&str>>();

        let sets_string = super::generate_sets_string(&columns, &mut current_param_index);

        if sets_string.is_empty() {
            return Ok(Status::NoContent);
        }

        let set_binds = vec![
            req.inventory_item_id
                .as_ref()
                .map(|v| SqlType::Int(v.clone())),
            req.quantity.as_ref().map(|v| SqlType::Int(v.clone())),
            req.price.as_ref().map(|v| SqlType::BigDecimal(v.clone())),
        ]
        .into_iter()
        .flatten();

        let query_str = format!(
            r#"
        UPDATE order_items
        SET {}
        WHERE order_id = ${}
        "#,
            sets_string, current_param_index
        );

        let query = sqlx::query(&query_str);

        let query = set_binds
            .into_iter()
            .fold(query.bind(id), |query, value| value.bind_to_query(query));

        let res = query.execute(&mut *transaction).await;

        if let Err(error) = res {
            transaction.rollback().await.map_err(|e| ApiError(
                Status::InternalServerError,
                format!("Failed to rollback transaction: {}", e),
            ))?;

            match error {
                sqlx::Error::RowNotFound => {
                    return Err(ApiError(
                        Status::BadRequest,
                        format!("Order with id {} not found", id),
                    ));
                }
                _ => return Err(error.into()),
            }
        }
    }

    transaction.commit().await.map_err(|e| ApiError(
        Status::InternalServerError,
        format!("Failed to commit transaction: {}", e),
    ))?;

    Ok(Status::NoContent)
}

#[rocket::delete("/orders/<id>")]
pub(super) async fn delete(
    id: i32,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
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
