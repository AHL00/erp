use sqlx::types::BigDecimal;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: BigDecimal,
    pub stock: i32,
    pub quantity_per_box: i32,
}