use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Acquire};

use crate::{
    db::{FromDB, DB},
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

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) struct Order {
    #[serde(flatten)]
    pub meta: OrderMeta,
    pub items: Vec<OrderItem>,
}

#[derive(FromRow, Debug)]
pub(super) struct OrderMetaRow {
    id: i32,
    date_time: chrono::DateTime<chrono::Utc>,
    customer: sqlx_core::types::Json<Customer>,
    created_by_user: sqlx_core::types::Json<UserRow>,
    amount_paid: sqlx::types::BigDecimal,
    retail: bool,
    notes: String,
}

pub(super) type OrderTotal = sqlx::types::BigDecimal;

impl From<OrderMetaRow> for OrderMeta {
    fn from(row: OrderMetaRow) -> Self {
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
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<rocket::serde::json::Json<OrderMeta>, ApiError> {
    let order_meta: OrderMeta = sqlx::query_as(
        r#"
        SELECT 
            orders.id,
            orders.date_time,
            orders.amount_paid,
            orders.retail,
            orders.notes,
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user
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
    .map(|row: OrderMetaRow| row.into())?;

    Ok(order_meta.into())
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct OrderItem {
    pub id: i32,
    pub inventory_item: InventoryItem,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

#[derive(FromRow, Debug, Deserialize)]
pub(super) struct OrderItemRow {
    pub id: i32,
    pub inventory: sqlx_core::types::Json<InventoryItem>,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

impl From<OrderItemRow> for OrderItem {
    fn from(value: OrderItemRow) -> Self {
        OrderItem {
            id: value.id,
            inventory_item: value.inventory.0,
            quantity: value.quantity,
            price: value.price,
        }
    }
}

#[rocket::get("/orders/<id>/items")]
pub(super) async fn get_items(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<OrderItem>>, ApiError> {
    let order_items: Vec<OrderItem> = sqlx::query_as(
        r#"
        SELECT 
            order_items.id as id,
            row_to_json(inventory) as inventory,
            order_items.price as price,
            order_items.quantity as quantity
        FROM order_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE order_id = $1
        "#,
    )
    .bind(id)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
    .map(|rows: Vec<OrderItemRow>| rows.into_iter().map(OrderItem::from).collect())?;

    Ok(rocket::serde::json::Json(order_items))
}

#[rocket::get("/orders/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
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
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
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
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user
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
        .map(|rows: Vec<OrderMetaRow>| rows.into_iter().map(OrderMeta::from).collect())?;

    Ok(rocket::serde::json::Json(orders))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderItemUpdateRequest {
    /// If the id is None, a new item will be created
    /// If the id is Some, the item with that id will be updated
    pub order_item_id: Option<i32>,
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

#[rocket::get("/orders/<id>/total")]
pub(super) async fn total(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>
) -> Result<rocket::serde::json::Json<OrderTotal>, ApiError> {
    Ok(rocket::serde::json::Json(get_order_total(id, &mut db).await?))
}

pub(super) async fn get_order_total(order_id: i32, db: &mut DB) -> Result<OrderTotal, ApiError> {
    let total: (sqlx::types::BigDecimal,) = sqlx::query_as(
        r#"
        SELECT sum(price * quantity) as total
        FROM order_items
        WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_one(&mut ***db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    Ok(total.0)
}

#[rocket::patch("/orders/<id>", data = "<req>")]
pub(super) async fn patch(
    id: i32,
    req: rocket::serde::json::Json<OrderPatchRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct StockUpdate {
    pub inventory_item: InventoryItem,
    pub delta: i32,
}

/// Post one or more items to an order
/// POST /orders/<id>/items
#[rocket::post("/orders/<id>/items/update", data = "<req>")]
pub(super) async fn update_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<OrderItemUpdateRequest>>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_WRITE as u32 }>,
) -> Result<ApiReturn<Vec<StockUpdate>>, ApiError> {
    let requests = req.into_inner();

    // Get current items for this order
    let current_items: Vec<OrderItem> = sqlx::query_as(
        r#"
        SELECT 
            order_items.id as id,
            row_to_json(inventory) as inventory,
            order_items.price as price,
            order_items.quantity as quantity
        FROM order_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE order_id = $1
        "#,
    )
    .bind(id)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            ApiError(Status::BadRequest, format!("Row with id {} not found", id))
        }
        _ => e.into(),
    })?
    .into_iter()
    .map(|row: OrderItemRow| row.into())
    .collect();

    log::info!("1");

    // Calculate stock deltas
    let mut stock_updates = vec![];

    for request in &requests {
        let current_item_match = current_items
            .iter()
            .find(|item| item.inventory_item.id == request.inventory_item_id);

        let delta = request.quantity - current_item_match.map(|item| item.quantity).unwrap_or(0);

        if delta != 0 {
            stock_updates.push(StockUpdate {
                inventory_item: InventoryItem::from_db(request.inventory_item_id, &mut db).await?,
                delta,
            });
        }
    }

    log::info!("2");

    let mut transaction = db.begin().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to start transaction: {}", e),
        )
    })?;

    for (i, req) in requests.into_iter().enumerate() {
        if let Some(order_item_id) = req.order_item_id {
            // Patch existing item
            let res = sqlx::query(
                r#"
            UPDATE order_items
            SET inventory_id = $1, quantity = $2, price = $3
            WHERE id = $4
            "#,
            )
            .bind(req.inventory_item_id)
            .bind(req.quantity)
            .bind(req.price)
            .bind(order_item_id)
            .execute(&mut *transaction)
            .await
            ;

            if let Err(error) = res {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!(
                        "Failed to update order item request at index {}: {}",
                        i, error
                    ),
                ));
            }

        log::info!("3");

        } else {
            // Do not allow two order items with the same item
            if current_items.iter().any(|item| item.inventory_item.id == req.inventory_item_id) {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::BadRequest,
                    format!(
                        "Order item with inventory item id {} already exists",
                        req.inventory_item_id
                    ),
                ));
            }

            // Insert new item
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
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!(
                        "Failed to insert order item request at index {}: {}",
                        i, error
                    ),
                ));
            }

            log::info!("4");
        }
    }

    // Update stock
    for update in stock_updates.iter() {
        let res = sqlx::query(
            r#"
        UPDATE inventory
        SET stock = stock + $1
        WHERE id = $2
        "#,
        )
        .bind(update.delta)
        .bind(update.inventory_item.id)
        .execute(&mut *transaction)
        .await       
        ;

        if let Err(error) = res {
            transaction.rollback().await.map_err(|e| {
                ApiError(
                    Status::InternalServerError,
                    format!("Failed to rollback transaction: {}", e),
                )
            })?;

            return Err(ApiError(
                Status::InternalServerError,
                format!("Failed to update stock: {}", error),
            ));
        }
    }

    transaction.commit().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    Ok(ApiReturn(Status::Created, stock_updates))
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