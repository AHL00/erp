use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, FromRow};

use crate::{
    db::{FromDB, DB},
    routes::{
        auth::{AuthGuard, UserRow},
        ListRequest,
    },
    types::permissions::UserPermissionEnum,
};

use super::{
    auth::{AuthCookie, User},
    customers::Customer,
    inventory::InventoryItem,
    search::SearchRequest,
    ApiError, ApiReturn, SqlType, StockUpdate, StockUpdateFactory,
};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct OrderMeta {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    /// This will be false if the order is retail
    pub customer: Option<Customer>,
    pub created_by_user: User,
    pub amount_paid: sqlx::types::BigDecimal,
    pub retail: bool,
    pub retail_customer_name: Option<String>,
    pub retail_customer_phone: Option<String>,
    pub retail_customer_address: Option<String>,
    pub fulfilled: bool,
    pub notes: String,
    pub total: sqlx::types::BigDecimal,
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
    customer: Option<sqlx::types::Json<Customer>>,
    created_by_user: sqlx::types::Json<UserRow>,
    amount_paid: sqlx::types::BigDecimal,
    retail: bool,
    retail_customer_name: Option<String>,
    retail_customer_phone: Option<String>,
    retail_customer_address: Option<String>,
    fulfilled: bool,
    notes: String,
    total: sqlx::types::BigDecimal,
}

pub(super) type OrderTotal = sqlx::types::BigDecimal;

impl From<OrderMetaRow> for OrderMeta {
    fn from(row: OrderMetaRow) -> Self {
        Self {
            id: row.id,
            date_time: row.date_time,
            customer: row.customer.map(|c| c.0),
            created_by_user: row.created_by_user.0.into(),
            amount_paid: row.amount_paid,
            fulfilled: row.fulfilled,
            retail: row.retail,
            retail_customer_name: row.retail_customer_name,
            retail_customer_phone: row.retail_customer_phone,
            retail_customer_address: row.retail_customer_address,
            notes: row.notes,
            total: row.total,
        }
    }
}

impl FromDB for OrderMeta {
    async fn from_db(id: i32, db: &mut DB) -> Result<Self, ApiError> {
        let order_meta: OrderMeta = sqlx::query_as(
            r#"
            SELECT 
                orders.id,
                orders.date_time,
                orders.amount_paid,
                orders.retail,
                orders.retail_customer_name,
                orders.retail_customer_phone,
                orders.retail_customer_address,
                orders.fulfilled,
                orders.notes,
                orders.total,
                row_to_json(customers) AS customer,
                row_to_json(users) AS created_by_user
            FROM orders
                LEFT JOIN customers ON orders.customer_id = customers.id
                INNER JOIN users ON orders.created_by_user_id = users.id
            WHERE orders.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut ***db)
        .await
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
        .map(|row: OrderMetaRow| row.into())?;

        Ok(order_meta.into())
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
            orders.retail_customer_name,
            orders.retail_customer_phone,
            orders.retail_customer_address,
            orders.fulfilled,
            orders.notes,
            orders.total,
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user
        FROM orders
            LEFT JOIN customers ON orders.customer_id = customers.id
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
    pub discount: sqlx::types::BigDecimal,
    pub discount_percentage: bool,
}

#[derive(FromRow, Debug, Deserialize)]
pub(super) struct OrderItemRow {
    pub id: i32,
    pub inventory: sqlx_core::types::Json<InventoryItem>,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
    pub discount: sqlx::types::BigDecimal,
    pub discount_percentage: bool,
}

impl From<OrderItemRow> for OrderItem {
    fn from(value: OrderItemRow) -> Self {
        OrderItem {
            id: value.id,
            inventory_item: value.inventory.0,
            quantity: value.quantity,
            price: value.price,
            discount: value.discount,
            discount_percentage: value.discount_percentage,
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
            order_items.quantity as quantity,
            order_items.discount as discount,
            order_items.discount_percentage as discount_percentage
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
    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM orders")
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
            orders.retail_customer_name,
            orders.retail_customer_phone,
            orders.retail_customer_address,
            orders.notes,
            orders.fulfilled,
            orders.total,
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user
        FROM orders
            LEFT JOIN customers ON orders.customer_id = customers.id
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

#[rocket::post("/orders/search", data = "<req>")]
pub(super) async fn search(
    req: rocket::serde::json::Json<SearchRequest>,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<OrderMeta>>, ApiError> {
    let req = req.into_inner();

    let x = req
        .column
        .unwrap_or(req.nested_access.unwrap_or("id".to_string()));

    let query_str = format!(
        r#"
        SELECT orders.id,
            orders.date_time,
            orders.amount_paid,
            orders.retail,
            orders.retail_customer_name,
            orders.retail_customer_phone,
            orders.retail_customer_address,
            orders.notes,
            orders.fulfilled,
            orders.total,
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user,
            word_similarity($1, {}::text) AS sml
        FROM orders
            LEFT JOIN customers ON orders.customer_id = customers.id
            INNER JOIN users ON orders.created_by_user_id = users.id
        WHERE $1 <% {}::text
        ORDER BY sml DESC, {}::text
        LIMIT $2
        "#,
        x, x, x
    );

    let query = sqlx::query_as(&query_str);

    let orders: Vec<OrderMeta> = query
        .bind(&req.search)
        .bind(req.count)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::ColumnNotFound(c) => {
                ApiError(Status::NotFound, format!("Column not found: {}", c))
            }
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
    pub discount: sqlx::types::BigDecimal,
    pub discount_percentage: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderPostRequest {
    pub customer_id: Option<i32>,
    pub retail: bool,
    pub retail_customer_name: Option<String>,
    pub retail_customer_phone: Option<String>,
    pub retail_customer_address: Option<String>,
    pub notes: String,
    pub amount_paid: sqlx::types::BigDecimal,
    pub fulfilled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct OrderPatchRequest {
    pub customer_id: Option<i32>,
    pub set_customer_id_null: bool,
    pub retail: Option<bool>,
    pub retail_customer_name: Option<String>,
    pub retail_customer_phone: Option<String>,
    pub retail_customer_address: Option<String>,
    pub set_retail_customer_null: bool,
    pub notes: Option<String>,
    pub amount_paid: Option<sqlx::types::BigDecimal>,
    pub fulfilled: Option<bool>,
    pub date_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[rocket::post("/orders", data = "<req>")]
pub(super) async fn post(
    req: rocket::serde::json::Json<OrderPostRequest>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_CREATE as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let req = req.into_inner();

    let user_id = auth.auth_info.user.id;

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO orders (customer_id, created_by_user_id, amount_paid, retail, retail_customer_name, retail_customer_phone, retail_customer_address, notes, fulfilled)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(req.customer_id)
    .bind(user_id)
    .bind(req.amount_paid)
    .bind(req.retail)
    .bind(req.retail_customer_name)
    .bind(req.retail_customer_phone)
    .bind(req.retail_customer_address)
    .bind(req.notes)
    .bind(req.fulfilled)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

#[rocket::get("/orders/<id>/total")]
pub(super) async fn total(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<rocket::serde::json::Json<OrderTotal>, ApiError> {
    Ok(rocket::serde::json::Json(
        get_order_total(id, &mut db).await?,
    ))
}

pub(super) async fn get_order_total(order_id: i32, db: &mut DB) -> Result<OrderTotal, ApiError> {
    let total: (OrderTotal,) = sqlx::query_as(
        r#"
        SELECT get_order_total($1)
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
    _auth: AuthGuard<{ UserPermissionEnum::ORDER_UPDATE as u32 }>,
) -> Result<Status, ApiError> {
    let req = req.into_inner();

    let mut transaction = db.begin().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to start transaction: {}", e),
        )
    })?;

    let mut current_param_index = 1;

    let columns = vec![
        req.customer_id.as_ref().map(|_| "customer_id"),
        req.retail.as_ref().map(|_| "retail"),
        req.retail_customer_name
            .as_ref()
            .map(|_| "retail_customer_name"),
        req.retail_customer_phone
            .as_ref()
            .map(|_| "retail_customer_phone"),
        req.retail_customer_address
            .as_ref()
            .map(|_| "retail_customer_address"),
        req.notes.as_ref().map(|_| "notes"),
        req.amount_paid.as_ref().map(|_| "amount_paid"),
        req.fulfilled.as_ref().map(|_| "fulfilled"),
        req.date_time.as_ref().map(|_| "date_time"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets_string = super::generate_sets_string(&columns, &mut current_param_index);

    if sets_string.is_empty() {
        return Ok(Status::NoContent);
    }

    let set_binds = vec![
        req.customer_id.as_ref().map(|v| SqlType::Int(v.clone())),
        req.retail.as_ref().map(|v| SqlType::Boolean(v.clone())),
        req.retail_customer_name
            .as_ref()
            .map(|v| SqlType::String(v.clone())),
        req.retail_customer_phone
            .as_ref()
            .map(|v| SqlType::String(v.clone())),
        req.retail_customer_address
            .as_ref()
            .map(|v| SqlType::String(v.clone())),
        req.notes.as_ref().map(|v| SqlType::String(v.clone())),
        req.amount_paid
            .as_ref()
            .map(|v| SqlType::BigDecimal(v.clone())),
        req.fulfilled.as_ref().map(|v| SqlType::Boolean(v.clone())),
        req.date_time.as_ref().map(|v| SqlType::DateTime(v.clone())),
    ]
    .into_iter()
    .flatten();

    let set_binds = set_binds.collect::<Vec<SqlType>>();

    let query_str = format!(
        r#"
        UPDATE orders
        SET {}
        WHERE id = ${}
        "#,
        sets_string, current_param_index
    );

    let mut query = sqlx::query(&query_str);

    query = set_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query(query));

    query = query.bind(id);

    query
        .execute(&mut *transaction)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError(
                Status::BadRequest,
                format!("Order with id {} not found", id),
            ),
            _ => e.into(),
        })?;

    // Inefficient, but it's the easiest way
    if req.set_customer_id_null {
        sqlx::query(
            r#"
            UPDATE orders
            SET customer_id = NULL
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError(
                Status::BadRequest,
                format!("Order with id {} not found", id),
            ),
            _ => e.into(),
        })?;
    }

    if req.set_retail_customer_null {
        sqlx::query(
            r#"
            UPDATE orders
            SET retail_customer_name = NULL, retail_customer_phone = NULL, retail_customer_address = NULL
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError(
                Status::BadRequest,
                format!("Order with id {} not found", id),
            ),
            _ => e.into(),
        })?;
    }

    transaction.commit().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    Ok(Status::NoContent)
}

async fn calculate_stock_deltas(
    requests: &Vec<OrderItemUpdateRequest>,
    current_items: &Vec<OrderItem>,
    order_id: i32,
    auth_info: &AuthCookie,
    db: &mut DB,
) -> Result<Vec<StockUpdateFactory>, ApiError> {
    let mut stock_update_factories = vec![];

    // Calculate stock deltas assuming that all current items are removed
    for item in current_items.iter() {
        stock_update_factories.push(StockUpdateFactory {
            inventory: item.inventory_item.clone(),
            delta: item.quantity,
            order_item_id: Some(item.id),
            order_id: Some(order_id),
            purchase_item_id: None,
            purchase_id: None,
            created_by_user_id: auth_info.user.id,
        });
    }

    // Calculate stock deltas for new items
    for req in requests.iter() {
        // Find and merge
        stock_update_factories
            .iter_mut()
            .find(|update| update.inventory.id == req.inventory_item_id)
            .map(|update| update.delta -= req.quantity);

        // Add new items
        if !current_items
            .iter()
            .any(|item| item.inventory_item.id == req.inventory_item_id)
        {
            stock_update_factories.push(StockUpdateFactory {
                inventory: InventoryItem::from_db(req.inventory_item_id, db).await?,
                delta: -req.quantity,
                // No item id because it's a new item. We don't know the id yet.
                order_item_id: None,
                order_id: Some(order_id),
                purchase_item_id: None,
                purchase_id: None,
                created_by_user_id: auth_info.user.id,
            });
        }
    }

    // Remove items with 0 delta
    stock_update_factories.retain(|update| update.delta != 0);

    Ok(stock_update_factories)
}

#[rocket::post("/orders/<id>/items/update/preview", data = "<req>")]
pub(super) async fn preview_update_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<OrderItemUpdateRequest>>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_READ as u32 }>,
) -> Result<ApiReturn<Vec<StockUpdateFactory>>, ApiError> {
    let requests = req.into_inner();

    // Check for duplicate inventory items
    let mut seen_item_ids = std::collections::HashSet::new();
    for req in requests.iter() {
        if seen_item_ids.contains(&req.inventory_item_id) {
            return Err(ApiError(
                Status::BadRequest,
                format!("Duplicate inventory item id {}", req.inventory_item_id),
            ));
        }

        seen_item_ids.insert(req.inventory_item_id);
    }

    // Get current items for this order
    let current_items: Vec<OrderItem> = sqlx::query_as(
        r#"
        SELECT 
            order_items.id as id,
            row_to_json(inventory) as inventory,
            order_items.price as price,
            order_items.quantity as quantity,
            order_items.discount as discount,
            order_items.discount_percentage as discount_percentage
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

    // Calculate stock deltas
    let stock_updates =
        calculate_stock_deltas(&requests, &current_items, id, &auth.auth_info, &mut db)
            .await
            .map_err(|e| {
                ApiError(
                    Status::InternalServerError,
                    format!("Failed to calculate stock deltas: {:?}", e),
                )
            })?;

    Ok(ApiReturn(Status::Ok, stock_updates))
}

/// Update all the items in an order
/// If an item is not in the request, it will be removed
/// If an item is in the request, it will be updated
/// If an item is not in the current items, it will be added
/// POST /orders/<id>/items
#[rocket::post("/orders/<id>/items/update", data = "<req>")]
pub(super) async fn update_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<OrderItemUpdateRequest>>,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_UPDATE as u32 }>,
) -> Result<ApiReturn<Vec<StockUpdate>>, ApiError> {
    let requests = req.into_inner();

    // Check for duplicate inventory items
    let mut inventory_item_ids = vec![];
    for req in requests.iter() {
        if inventory_item_ids.contains(&req.inventory_item_id) {
            return Err(ApiError(
                Status::BadRequest,
                format!("Duplicate inventory item id {}", req.inventory_item_id),
            ));
        }

        inventory_item_ids.push(req.inventory_item_id);
    }

    // Get current items for this order
    let current_items: Vec<OrderItem> = sqlx::query_as(
        r#"
        SELECT 
            order_items.id as id,
            row_to_json(inventory) as inventory,
            order_items.price as price,
            order_items.quantity as quantity,
            order_items.discount as discount,
            order_items.discount_percentage as discount_percentage
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

    // Calculate stock deltas
    let stock_update_factories =
        calculate_stock_deltas(&requests, &current_items, id, &auth.auth_info, &mut db)
            .await
            .map_err(|e| {
                ApiError(
                    Status::InternalServerError,
                    format!("Failed to calculate stock deltas: {:?}", e),
                )
            })?;

    let mut transaction = db.begin().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to start transaction: {}", e),
        )
    })?;

    for (i, req) in requests.iter().enumerate() {
        if let Some(order_item_id) = req.order_item_id {
            // Check if is different to existing
            let current = current_items
                .iter()
                .find(|item| item.id == order_item_id)
                .ok_or_else(|| {
                    ApiError(
                        Status::BadRequest,
                        format!("Order item with id {} not found", order_item_id),
                    )
                })?;

            let update = current.inventory_item.id != req.inventory_item_id
                || current.quantity != req.quantity
                || current.price != req.price
                || current.discount != req.discount
                || current.discount_percentage != req.discount_percentage;

            if !update {
                log::info!("Skipping: {:#?}", req);
                continue;
            }

            log::info!("Patching: {:#?}", req);

            // Patch existing item
            let res = sqlx::query(
                r#"
            UPDATE order_items
            SET inventory_id = $1, quantity = $2, price = $3, discount = $4, discount_percentage = $5
            WHERE id = $6
            "#,
            )
            .bind(req.inventory_item_id)
            .bind(req.quantity)
            .bind(req.price.clone())
            .bind(req.discount.clone())
            .bind(req.discount_percentage)
            .bind(order_item_id)
            .execute(&mut *transaction)
            .await;

            log::info!("Patch result: {:#?}", res);

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
            } else if let Ok(res) = res {
                if res.rows_affected() == 0 {
                    transaction.rollback().await.map_err(|e| {
                        ApiError(
                            Status::InternalServerError,
                            format!("Failed to rollback transaction: {}", e),
                        )
                    })?;

                    return Err(ApiError(
                        Status::BadRequest,
                        format!(
                            "Order item with id {} not found in order with id {}",
                            order_item_id, id
                        ),
                    ));
                }
            }
        } else {
            // Do not allow two order items with the same item
            if current_items
                .iter()
                .any(|item| item.inventory_item.id == req.inventory_item_id)
            {
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
                INSERT INTO order_items (order_id, inventory_id, quantity, price, discount, discount_percentage)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id
                "#,
            )
            .bind(id)
            .bind(req.inventory_item_id)
            .bind(req.quantity)
            .bind(req.price.clone())
            .bind(req.discount.clone())
            .bind(req.discount_percentage)
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
        }
    }

    // If there are items in the current
    // order items that are not in the request,
    // remove them
    for current_item in &current_items {
        if !requests.iter().any(|req| {
            if let Some(order_item_id) = req.order_item_id {
                order_item_id == current_item.id
            } else {
                false
            }
        }) {
            log::info!("Deleting item: {:#?}", current_item);
            let res = sqlx::query(
                r#"
            DELETE FROM order_items
            WHERE id = $1
            "#,
            )
            .bind(current_item.id)
            .execute(&mut *transaction)
            .await;

            if let Err(error) = res {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!("Failed to delete order item: {}", error),
                ));
            }
        }
    }

    // Update stock
    let mut stock_updates = vec![];
    for update_factory in stock_update_factories.iter() {
        let res = sqlx::query(
            r#"
        UPDATE inventory
        SET stock = stock + $1
        WHERE id = $2
        "#,
        )
        .bind(update_factory.delta)
        .bind(update_factory.inventory.id)
        .execute(&mut *transaction)
        .await;

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

        // Update stock update history
        let stock_update: StockUpdate = sqlx::query_as(
            r#"
            INSERT INTO stock_updates (inventory_id, created_by_user_id, delta, order_item_id, order_id, purchase_item_id, purchase_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(update_factory.inventory.id)
        .bind(update_factory.created_by_user_id)
        .bind(update_factory.delta)
        .bind(update_factory.order_item_id)
        .bind(update_factory.order_id)
        .bind(update_factory.purchase_item_id)
        .bind(update_factory.purchase_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| {
            ApiError(
                Status::InternalServerError,
                format!("Failed to insert stock update: {}", e),
            )
        })?;

        stock_updates.push(stock_update);
    }

    transaction.commit().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    Ok(ApiReturn(Status::Ok, stock_updates))
}

#[rocket::delete("/orders/<id>")]
pub(super) async fn delete(
    id: i32,
    mut db: DB,
    auth: AuthGuard<{ UserPermissionEnum::ORDER_DELETE as u32 }>,
) -> Result<ApiReturn<Vec<StockUpdate>>, ApiError> {
    // Add stock back for all items in the order
    let order_items: Vec<OrderItem> = sqlx::query_as(
        r#"
    SELECT 
        order_items.id as id,
        row_to_json(inventory) as inventory,
        order_items.price as price,
        order_items.quantity as quantity
        order_items.discount as discount,
        order_items.discount_percentage as discount_percentage
    FROM order_items
        INNER JOIN inventory ON inventory_id = inventory.id
    WHERE order_id = $1
    "#,
    )
    .bind(id)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError(Status::NotFound, format!("No order with id {}", id)),
        _ => e.into(),
    })?
    .into_iter()
    .map(|row: OrderItemRow| row.into())
    .collect();

    let stock_update_factories =
        calculate_stock_deltas(&vec![], &order_items, id, &auth.auth_info, &mut db)
            .await
            .map_err(|e| {
                ApiError(
                    Status::InternalServerError,
                    format!("Failed to calculate stock deltas: {:?}", e),
                )
            })?;

    // Start transaction
    let mut transaction = db.begin().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to start transaction: {}", e),
        )
    })?;

    // Drop all order_items
    sqlx::query(
        r#"
        DELETE FROM order_items
        WHERE order_id = $1
        "#,
    )
    .bind(id)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM orders
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&mut *transaction)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError(Status::NotFound, format!("No order with id {}", id)),
        _ => e.into(),
    })?;

    // Add stock back
    let mut stock_updates = vec![];
    for update_factory in stock_update_factories {
        let res = sqlx::query(
            r#"
            UPDATE inventory
            SET stock = stock + $1
            WHERE id = $2
            "#,
        )
        .bind(update_factory.delta)
        .bind(update_factory.inventory.id)
        .execute(&mut *transaction)
        .await;

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

        // Update stock update history
        let stock_update: StockUpdate = sqlx::query_as(
            r#"
            INSERT INTO stock_updates (inventory_id, created_by_user_id, delta, order_item_id, order_id, purchase_item_id, purchase_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(update_factory.inventory.id)
        .bind(update_factory.created_by_user_id)
        .bind(update_factory.delta)
        .bind(update_factory.order_item_id)
        .bind(update_factory.order_id)
        .bind(update_factory.purchase_item_id)
        .bind(update_factory.purchase_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| {
            ApiError(
                Status::InternalServerError,
                format!("Failed to insert stock update: {}", e),
            )
        })?;

        stock_updates.push(stock_update);
    }

    transaction.commit().await.map_err(|e| {
        ApiError(
            Status::InternalServerError,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    Ok(ApiReturn(Status::Ok, stock_updates))
}
