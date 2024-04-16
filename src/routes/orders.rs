use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{db::DB, routes::auth::AuthGuard, types::permissions::UserPermissionEnum};

use super::{auth::User, customers::Customer, inventory::InventoryItem, ApiError};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
pub(super) struct OrderMeta {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    /// This will be false if the order is retail
    pub customer: Option<Customer>,
    pub created_by_user: User,
    pub amount_paid: sqlx::types::BigDecimal,
    pub retail: bool,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
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
    pub meta: OrderMeta,
    pub items: Vec<OrderItem>,
}


#[rocket::get("/orders/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::ADMIN as u32 }>,
) -> Result<rocket::serde::json::Json<Order>, ApiError> {   
    let row = sqlx::query(
        r#"
        SELECT get_order($1)
        "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await
    .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;

    let json = row.try_get_raw(0)?.as_str().map_err(
        |e| ApiError(Status::InternalServerError, e.to_string())
    )?;

    let order = serde_json::from_str::<Order>(json)
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))?;
    
    // Re-encoding json may not be efficient but we may do some serde flattening later.
    // Also allows some error handling to be done in the future.
    // It also makes sure that the json layout is correct before sending it to the client.
    // If not perfomant enough, we can always just send the json variable directly.
    Ok(rocket::serde::json::Json(order))
}
