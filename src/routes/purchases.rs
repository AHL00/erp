use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, FromRow};

use crate::{db::FromDB, routes::SqlType, types::permissions::UserPermissionEnum};

use super::{
    auth::{AuthCookie, AuthGuard, User, UserRow},
    public::InventoryItem,
    suppliers::Supplier,
    ApiError, ApiReturn, StockUpdate, StockUpdateFactory,
};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct PurchaseMeta {
    pub id: i32,
    pub created_by_user: User,
    pub supplier: Supplier,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub amount_paid: sqlx::types::BigDecimal,
    pub notes: String,
}

#[derive(FromRow, Debug)]
pub(super) struct PurchaseMetaRow {
    pub id: i32,
    pub created_by_user: sqlx::types::Json<UserRow>,
    pub supplier: sqlx::types::Json<Supplier>,
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub amount_paid: sqlx::types::BigDecimal,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct PurchaseItem {
    pub id: i32,
    pub inventory_item: InventoryItem,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

#[derive(FromRow, Debug, Deserialize)]
pub(super) struct PurchaseItemRow {
    pub id: i32,
    pub inventory: sqlx::types::Json<InventoryItem>,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

impl From<PurchaseItemRow> for PurchaseItem {
    fn from(row: PurchaseItemRow) -> Self {
        Self {
            id: row.id,
            inventory_item: row.inventory.0.into(),
            quantity: row.quantity,
            price: row.price,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Purchase {
    #[serde(flatten)]
    pub meta: PurchaseMeta,
    pub items: Vec<PurchaseItem>,
}

pub(super) type PurchaseTotal = sqlx::types::BigDecimal;

impl From<PurchaseMetaRow> for PurchaseMeta {
    fn from(row: PurchaseMetaRow) -> Self {
        Self {
            id: row.id,
            created_by_user: row.created_by_user.0.into(),
            supplier: row.supplier.0.into(),
            date_time: row.date_time,
            amount_paid: row.amount_paid,
            notes: row.notes,
        }
    }
}

#[rocket::get("/purchases/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: crate::db::DB,
    _auth: crate::routes::auth::AuthGuard<
        { crate::types::permissions::UserPermissionEnum::PURCHASE_READ as u32 },
    >,
) -> Result<rocket::serde::json::Json<PurchaseMeta>, ApiError> {
    let row: PurchaseMeta = sqlx::query_as(
        r#"
            SELECT
                purchases.id,
                purchases.date_time,
                purchases.amount_paid,
                purchases.notes,
                row_to_json(suppliers) AS supplier,
                row_to_json(users) AS created_by_user
            FROM purchases
                LEFT JOIN suppliers ON purchases.supplier_id = suppliers.id
                LEFT JOIN users ON purchases.created_by_user_id = users.id
            WHERE purchases.id = $1
            "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
    .map(|row: PurchaseMetaRow| row.into())?;

    Ok(rocket::serde::json::Json(row.into()))
}

#[rocket::get("/purchases/<id>/items")]
pub(super) async fn get_items(
    id: i32,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_READ as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<PurchaseItem>>, ApiError> {
    let purchase_items: Vec<PurchaseItem> = sqlx::query_as(
        r#"
        SELECT 
            purchase_items.id as id,
            row_to_json(inventory) as inventory,
            purchase_items.price as price,
            purchase_items.quantity as quantity
        FROM purchase_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE purchase_id = $1
        "#,
    )
    .bind(id)
    .fetch_all(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
    .map(|rows: Vec<PurchaseItemRow>| rows.into_iter().map(PurchaseItem::from).collect())?;

    Ok(rocket::serde::json::Json(purchase_items))
}

#[rocket::get("/purchases/count")]
pub(super) async fn count(
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_READ as u32 }>,
) -> Result<rocket::serde::json::Json<i64>, ApiError> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM purchases")
        .fetch_one(&mut **db)
        .await
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    Ok(rocket::serde::json::Json(count))
}

#[rocket::post("/purchases/list", data = "<req>")]
pub(super) async fn list(
    req: rocket::serde::json::Json<crate::routes::public::ListRequest>,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_READ as u32 }>,
) -> Result<rocket::serde::json::Json<Vec<PurchaseMeta>>, ApiError> {
    let req = req.into_inner();

    let mut current_param = 1;

    let sorts_string = super::generate_sorts_string(&req.sorts);

    let (filters_string, filter_binds) =
        super::generate_filters_string(&req.filters, &mut current_param);

    let query_str = format!(
        r#"
        SELECT
            purchases.id,
            purchases.date_time,
            purchases.amount_paid,
            purchases.notes,
            row_to_json(suppliers) AS supplier,
            row_to_json(users) AS created_by_user
        FROM purchases
            LEFT JOIN suppliers ON purchases.supplier_id = suppliers.id
            LEFT JOIN users ON purchases.created_by_user_id = users.id
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

    let purchases: Vec<PurchaseMeta> = query
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::NotFound, "No purchases found".to_string())
            }
            e => ApiError(Status::InternalServerError, e.to_string()),
        })
        .map(|rows: Vec<PurchaseMetaRow>| rows.into_iter().map(Into::into).collect())?;

    Ok(rocket::serde::json::Json(purchases))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct PurchaseItemUpdateRequest {
    /// The ID of the inventory item
    /// If this is set, the inventory item will be updated
    /// If this is not set, the inventory item will be created
    pub purchase_item_id: Option<i32>,
    pub inventory_item_id: i32,
    pub quantity: i32,
    pub price: sqlx::types::BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct PurchasePostRequest {
    pub supplier_id: i32,
    pub notes: String,
    pub amount_paid: sqlx::types::BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct PurchasePatchRequest {
    pub supplier_id: Option<i32>,
    pub notes: Option<String>,
    pub amount_paid: Option<sqlx::types::BigDecimal>,
    pub date_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[rocket::post("/purchases", data = "<req>")]
pub(super) async fn post(
    req: rocket::serde::json::Json<PurchasePostRequest>,
    mut db: crate::db::DB,
    auth: AuthGuard<{ UserPermissionEnum::PURCHASE_WRITE as u32 }>,
) -> Result<ApiReturn<i32>, ApiError> {
    let req = req.into_inner();

    let id: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO purchases (supplier_id, created_by_user_id, notes, amount_paid)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(req.supplier_id)
    .bind(auth.auth_info.user.id)
    .bind(req.notes)
    .bind(req.amount_paid)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

#[rocket::get("/purchases/<id>/total")]
pub(super) async fn total(
    id: i32,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_READ as u32 }>,
) -> Result<rocket::serde::json::Json<PurchaseTotal>, ApiError> {
    Ok(rocket::serde::json::Json(
        get_purchase_total(id, &mut db).await?,
    ))
}

pub(super) async fn get_purchase_total(
    purchase_id: i32,
    db: &mut crate::db::DB,
) -> Result<PurchaseTotal, ApiError> {
    let total: (PurchaseTotal,) = sqlx::query_as(
        r#"
        SELECT SUM(price * quantity) FROM purchase_items WHERE purchase_id = $1
        "#,
    )
    .bind(purchase_id)
    .fetch_one(&mut ***db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    Ok(total.0)
}

#[rocket::patch("/purchases/<id>", data = "<req>")]
pub(super) async fn patch(
    id: i32,
    req: rocket::serde::json::Json<PurchasePatchRequest>,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_WRITE as u32 }>,
) -> Result<ApiReturn<()>, ApiError> {
    let req = req.into_inner();

    let mut current_param_index = 1;

    let columns = vec![
        req.supplier_id.as_ref().map(|_| "supplier_id"),
        req.notes.as_ref().map(|_| "notes"),
        req.amount_paid.as_ref().map(|_| "amount_paid"),
        req.date_time.as_ref().map(|_| "date_time"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets_string = super::generate_sets_string(&columns, &mut current_param_index);

    if sets_string.is_empty() {
        return Ok(ApiReturn(Status::Ok, ()));
    }

    let set_binds = vec![
        req.supplier_id.as_ref().map(|v| SqlType::Int(*v)),
        req.notes.as_ref().map(|v| SqlType::String(v.clone())),
        req.amount_paid
            .as_ref()
            .map(|v| SqlType::BigDecimal(v.clone())),
        req.date_time
            .as_ref()
            .map(|v| SqlType::DateTime(v.clone())),
    ]
    .into_iter()
    .flatten();

    let set_binds = set_binds.collect::<Vec<SqlType>>();

    let query_str = format!(
        r#"
        UPDATE purchases
        SET {}
        WHERE id = ${}
        "#,
        sets_string, current_param_index
    );

    let query = sqlx::query(&query_str);

    let mut query = set_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query(query));

    query = query.bind(id);

    query.execute(&mut **db).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => ApiError(
            Status::BadRequest,
            format!("Purchase with id {} not found", id),
        ),
        _ => e.into(),
    })?;

    Ok(ApiReturn(Status::Ok, ()))
}

async fn calculate_stock_deltas(
    requests: &Vec<PurchaseItemUpdateRequest>,
    current_items: &Vec<PurchaseItem>,
    purchase_id: i32,
    auth_info: &AuthCookie,
    db: &mut crate::db::DB,
) -> Result<Vec<StockUpdateFactory>, ApiError> {
    let mut stock_update_factories = vec![];

    for item in current_items.iter() {
        stock_update_factories.push(StockUpdateFactory {
            inventory: item.inventory_item.clone(),
            delta: -item.quantity,
            order_item_id: None,
            order_id: None,
            purchase_item_id: Some(item.id),
            purchase_id: Some(purchase_id),
            created_by_user_id: auth_info.user.id,
        });
    }

    // Calculate stock deltas for new items
    for req in requests.iter() {
        // Find and merge
        stock_update_factories
            .iter_mut()
            .find(|update| update.inventory.id == req.inventory_item_id)
            .map(|update| update.delta += req.quantity);

        // Add new items
        if !current_items
            .iter()
            .any(|item| item.inventory_item.id == req.inventory_item_id)
        {
            stock_update_factories.push(StockUpdateFactory {
                inventory: InventoryItem::from_db(req.inventory_item_id, db).await?,
                delta: req.quantity,
                order_item_id: None,
                order_id: None,
                // No purchase item id because it's a new item.
                purchase_item_id: None,
                purchase_id: Some(purchase_id),
                created_by_user_id: auth_info.user.id,
            });
        }
    }

    // Remove items with 0 deltas
    stock_update_factories.retain(|update| update.delta != 0);

    Ok(stock_update_factories)
}

#[rocket::post("/purchases/<id>/items/update/preview", data = "<req>")]
pub(super) async fn preview_update_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<PurchaseItemUpdateRequest>>,
    mut db: crate::db::DB,
    auth: AuthGuard<{ UserPermissionEnum::PURCHASE_WRITE as u32 }>,
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

    // Get current items in the purchase
    let current_items: Vec<PurchaseItem> = sqlx::query_as(
        r#"
        SELECT 
            purchase_items.id as id,
            row_to_json(inventory) as inventory,
            purchase_items.price as price,
            purchase_items.quantity as quantity
        FROM purchase_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE purchase_id = $1
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
    .map(|row: PurchaseItemRow| row.into())
    .collect();

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

/// Update all items in a purchase
/// If an item is not in the request, it will be removed
/// If an item is in the request, it will be updated
/// If an item is not in the current items, it will be added
#[rocket::post("/purchases/<id>/items/update", data = "<req>")]
pub(super) async fn update_items(
    id: i32,
    req: rocket::serde::json::Json<Vec<PurchaseItemUpdateRequest>>,
    mut db: crate::db::DB,
    auth: AuthGuard<{ UserPermissionEnum::PURCHASE_WRITE as u32 }>,
) -> Result<ApiReturn<()>, ApiError> {
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

    // Get current items in the purchase
    let current_items: Vec<PurchaseItem> = sqlx::query_as(
        r#"
        SELECT 
            purchase_items.id as id,
            row_to_json(inventory) as inventory,
            purchase_items.price as price,
            purchase_items.quantity as quantity
        FROM purchase_items
            INNER JOIN inventory ON inventory_id = inventory.id
        WHERE purchase_id = $1
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
    .map(|row: PurchaseItemRow| row.into())
    .collect();

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
            format!("Failed to start transaction: {:?}", e),
        )
    })?;

    for (i, req) in requests.iter().enumerate() {
        if let Some(purchase_item_id) = req.purchase_item_id {
            // Patch existing item
            let res = sqlx::query(
                r#"
                UPDATE purchase_items
                SET inventory_id = $1, quantity = $2, price = $3
                WHERE id = $4
                "#,
            )
            .bind(req.inventory_item_id)
            .bind(req.quantity)
            .bind(req.price.clone())
            .bind(purchase_item_id)
            .execute(&mut *transaction)
            .await;

            if let Err(e) = res {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {:?}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!(
                        "Failed to update purchase item request at index {}: {}",
                        i, e
                    ),
                ));
            }
        } else {
            // Create new item
            // Do not allow duplicate inventory items
            if current_items
                .iter()
                .any(|item| item.inventory_item.id == req.inventory_item_id)
            {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {:?}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::BadRequest,
                    format!(
                        "Duplicate inventory item id {} already exists",
                        req.inventory_item_id
                    ),
                ));
            }

            // Insert new item
            let id: Result<(i32,), _> = sqlx::query_as(
                r#"
                INSERT INTO purchase_items (purchase_id, inventory_id, quantity, price)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
            )
            .bind(id)
            .bind(req.inventory_item_id)
            .bind(req.quantity)
            .bind(req.price.clone())
            .fetch_one(&mut *transaction)
            .await;

            if let Err(e) = id {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {:?}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!(
                        "Failed to insert purchase item request at index {}: {}",
                        i, e
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
            if let Some(purchase_item_id) = req.purchase_item_id {
                purchase_item_id == current_item.id
            } else {
                false
            }
        }) {
            log::info!("Removing purchase item {}", current_item.id);
            let res = sqlx::query(
                r#"
            DELETE FROM purchase_items
            WHERE id = $1
            "#,
            )
            .bind(current_item.id)
            .execute(&mut *transaction)
            .await;

            if let Err(e) = res {
                transaction.rollback().await.map_err(|e| {
                    ApiError(
                        Status::InternalServerError,
                        format!("Failed to rollback transaction: {:?}", e),
                    )
                })?;

                return Err(ApiError(
                    Status::InternalServerError,
                    format!("Failed to delete purchase item {}: {}", current_item.id, e),
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

    Ok(ApiReturn(Status::Ok, ()))
}

#[rocket::delete("/purchases/<id>")]
pub(super) async fn delete(
    id: i32,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PURCHASE_WRITE as u32 }>,
) -> Result<Status, ApiError> {
    sqlx::query("DELETE FROM purchases WHERE id = $1")
        .bind(id)
        .execute(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::NotFound, format!("No order with id {}", id))
            }
            _ => e.into(),
        })?;

    Ok(Status::NoContent)
}
