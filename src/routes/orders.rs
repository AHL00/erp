use sqlx::types::chrono;

struct OrderMeta {
    pub id: i32,
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub customer_id: i32,
    pub created_by_user_id: i32,
}

