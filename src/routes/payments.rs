use bigdecimal::BigDecimal;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{db::FromDB, types::permissions::UserPermissionEnum};

use super::{
    auth::{AuthGuard, User, UserRow},
    customers::Customer,
    orders::{Order, OrderMeta},
    suppliers::Supplier,
    ApiError, ListRequest,
};

#[derive(FromRow, Debug)]
pub(super) struct PaymentRow {
    pub id: i32,
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub amount: BigDecimal,
    // JSON type <PaymentParty variants>
    pub party: sqlx::types::JsonValue,
    pub party_type: PartyType,
    pub transfer_type: TransferType,
    pub method: PaymentMethod,
    pub method_details: String,
    pub notes: String,
    pub created_by_user: sqlx::types::Json<UserRow>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "payment_method_t")]
#[ts(export)]
pub enum PaymentMethod {
    Cash,
    Bank,
    Mobile,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "party_type_t")]
pub enum PartyType {
    Customer,
    Supplier,
    Retail,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "transfer_type_t")]
#[ts(export)]

pub enum TransferType {
    Incoming,
    Outgoing,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Payment {
    pub id: i32,
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub amount: BigDecimal,
    pub party: PaymentParty,
    pub transfer_type: TransferType,
    pub method: PaymentMethod,
    pub method_details: String,
    pub notes: String,
    pub created_by_user: User,
}

impl Payment {
    async fn from_row(row: PaymentRow, db: &mut crate::db::DB) -> Result<Self, ApiError> {
        Ok(Self {
            id: row.id,
            date_time: row.date_time,
            amount: row.amount,
            party: match row.party_type {
                PartyType::Customer => PaymentParty::Customer(serde_json::from_value(row.party)?),
                PartyType::Supplier => PaymentParty::Supplier(serde_json::from_value(row.party)?),
                PartyType::Retail => PaymentParty::Retail(serde_json::from_value(row.party)?),
            },
            transfer_type: row.transfer_type,
            method: row.method,
            method_details: row.method_details,
            notes: row.notes,
            created_by_user: row.created_by_user.0.into(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) enum PaymentParty {
    Customer(Customer),
    Supplier(Supplier),
    Retail(OrderMeta),
}

#[rocket::get("/payments/<id>")]
pub(super) async fn get(
    id: i32,
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PAYMENT_READ as u32 }>,
) -> Result<Json<Payment>, ApiError> {
    let row = sqlx::query_as(
        r#"
            SELECT
                payments.id,
                payments.date_time,
                payments.amount,
                payments.party_id,
                payments.party_type,
                case 
                    when party_type = 'CUSTOMER' then row_to_json(customers)
                    when party_type = 'SUPPLIER' then row_to_json(suppliers)
                    when party_type = 'RETAIL' then row_to_json(orders)
                end as party,
                payments.transfer_type,
                payments.method,
                payments.method_details,
                payments.notes,
                row_to_json(users) AS created_by_user
            FROM payments
                INNER JOIN users ON payments.created_by_user_id = users.id
                LEFT JOIN customers ON payments.party_id = customers.id AND payments.party_type = 'CUSTOMER'
                LEFT JOIN suppliers ON payments.party_id = suppliers.id AND payments.party_type = 'SUPPLIER'
                LEFT JOIN orders ON payments.party_id = orders.id AND payments.party_type = 'RETAIL'
            WHERE payments.id = $1
            "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await?;

    Payment::from_row(row, &mut db).await.map(Json)
}

#[rocket::get("/payments/count")]
pub(super) async fn count(
    mut db: crate::db::DB,
    _auth: AuthGuard<{ UserPermissionEnum::PAYMENT_READ as u32 }>,
) -> Result<Json<i64>, ApiError> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM payments")
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(count))
}

// #[rocket::post("/payments/list", data = "<list_request>")]
// pub(super) async fn list(
//     mut db: crate::db::DB,
//     _auth: AuthGuard<{ UserPermissionEnum::EXPENSES_READ as u32 }>,
//     list_request: rocket::serde::json::Json<ListRequest>,
// ) -> Result<rocket::serde::json::Json<Vec<Payment>>, ApiError> {
//     let list_request = list_request.into_inner();

//     let mut current_param = 1;

//     let sorts_sql = super::generate_sorts_string(&list_request.sorts);

//     let (filters_sql, filter_binds) =
//         super::generate_filters_string(&list_request.filters, &mut current_param);

//     let query = format!(
//         r#"SELECT 
//             expenses.id, 
//             date_time, 
//             description, 
//             row_to_json(users) AS created_by_user,
//             amount
//         FROM expenses
//             INNER JOIN users ON expenses.created_by_user_id = users.id
//         {}
//         {}
//         LIMIT ${}
//         OFFSET ${}
//         "#,
//         filters_sql,
//         sorts_sql,
//         current_param,
//         current_param + 1
//     );

//     let query = sqlx::query_as(&query);

//     let query = filter_binds
//         .into_iter()
//         .fold(query, |query, value| value.bind_to_query_as(query));

//     let query = query
//         .bind(list_request.range.count)
//         .bind(list_request.range.offset);

//     let expenses: Vec<Expense> = query
//         .fetch_all(&mut **db)
//         .await
//         .map_err(|e| match e {
//             sqlx::Error::RowNotFound => ApiError(Status::NotFound, "No expenses found".to_string()),
//             _ => ApiError(Status::InternalServerError, e.to_string()),
//         })
//         .map(|rows: Vec<ExpenseRow>| rows.into_iter().map(Expense::from).collect())?;

//     Ok(rocket::serde::json::Json(expenses))
// }
