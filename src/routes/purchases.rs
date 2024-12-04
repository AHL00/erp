use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::{auth::User, suppliers::Supplier};



#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct PurchaseMeta {
    pub id: i32,
    pub created_by_user: User,
    pub supplier: Supplier,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
}