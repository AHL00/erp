use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;


#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Customer {
    id: i32,
    name: String,
    phone: String,
    address: String,
    notes: String,
}
