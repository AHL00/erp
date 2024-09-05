use rocket::http::Status;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
    db::DB,
    routes::{
        auth::{AuthGuard, UserRow},
        ListRequest,
    },
    types::permissions::UserPermissionEnum,
};

use super::{auth::User, ApiError, ApiReturn, SqlType};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, FromRow)]
#[ts(export)]
pub(super) struct Expense {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub created_by_user: User,
    pub amount: sqlx::types::BigDecimal,
}

#[derive(FromRow, Debug)]
pub(super) struct ExpenseRow {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub created_by_user: sqlx_core::types::Json<UserRow>,
    pub amount: sqlx::types::BigDecimal,
}

impl From<ExpenseRow> for Expense {
    fn from(row: ExpenseRow) -> Self {
        Self {
            id: row.id,
            date_time: row.date_time,
            description: row.description,
            created_by_user: row.created_by_user.0.into(),
            amount: row.amount,
        }
    }
}

#[rocket::get("/expenses/<id>")]
pub(super) async fn get(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::EXPENSES_READ as u32 }>,
    id: i32,
) -> Result<rocket::serde::json::Json<Expense>, ApiError> {
    let expense_row: ExpenseRow = sqlx::query_as(
        r#"
    SELECT 
        expenses.id, 
        date_time, 
        description, 
        row_to_json(users) AS created_by_user,
        amount
    FROM expenses
        INNER JOIN users ON expenses.created_by_user_id = users.id
    WHERE expenses.id = $1
    "#,
    )
    .bind(id)
    .fetch_one(&mut **db)
    .await?;

    Ok(rocket::serde::json::Json::from(Expense::from(expense_row)))
}

#[rocket::get("/expenses/count")]
pub(super) async fn count(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::EXPENSES_READ as u32 }>,
) -> Result<rocket::serde::json::Json<i64>, ApiError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM expenses")
        .fetch_one(&mut **db)
        .await?;

    Ok(rocket::serde::json::Json(count.0))
}

#[rocket::post("/expenses/list", data = "<list_request>")]
pub(super) async fn list(
    mut db: DB,
    _auth: AuthGuard<{ UserPermissionEnum::EXPENSES_READ as u32 }>,
    list_request: rocket::serde::json::Json<ListRequest>,
) -> Result<rocket::serde::json::Json<Vec<Expense>>, ApiError> {
    let list_request = list_request.into_inner();

    let mut current_param = 1;

    let sorts_sql = super::generate_sorts_string(&list_request.sorts);

    let (filters_sql, filter_binds) =
        super::generate_filters_string(&list_request.filters, &mut current_param);

    let query = format!(
        r#"SELECT 
            expenses.id, 
            date_time, 
            description, 
            row_to_json(users) AS created_by_user,
            amount
        FROM expenses
            INNER JOIN users ON expenses.created_by_user_id = users.id
        {}
        {}
        LIMIT ${}
        OFFSET ${}
        "#,
        filters_sql,
        sorts_sql,
        current_param,
        current_param + 1
    );

    let query = sqlx::query_as(&query);

    let query = filter_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query_as(query));

    let query = query
        .bind(list_request.range.count)
        .bind(list_request.range.offset);

    let expenses: Vec<Expense> = query
        .fetch_all(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError(Status::NotFound, "No expenses found".to_string()),
            _ => ApiError(Status::InternalServerError, e.to_string()),
        })
        .map(|rows: Vec<ExpenseRow>| rows.into_iter().map(Expense::from).collect())?;

    Ok(rocket::serde::json::Json(expenses))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct ExpensePostRequest {
    pub description: String,
    pub amount: sqlx::types::BigDecimal,
}

#[rocket::post("/expenses", data = "<expense>")]
pub(super) async fn post(
    mut db: DB,
    #[allow(unused)]
    auth: AuthGuard<{ UserPermissionEnum::EXPENSES_WRITE as u32 }>,
    expense: rocket::serde::json::Json<ExpensePostRequest>,
) -> Result<ApiReturn<i32>, ApiError> {
    let req = expense.into_inner();

    let id: (i32,) = sqlx::query_as(
        r#"
    INSERT INTO expenses (description, amount, created_by_user_id)
    VALUES ($1, $2, $3)
    RETURNING id
    "#,
    )
    .bind(&req.description)
    .bind(&req.amount)
    .bind(auth.auth_info.user.id)
    .fetch_one(&mut **db)
    .await?;

    Ok(ApiReturn(Status::Created, id.0))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct ExpensePatchRequest {
    pub description: Option<String>,
    pub amount: Option<sqlx::types::BigDecimal>,
}

#[rocket::patch("/expenses/<id>", data = "<expense>")]
pub(super) async fn patch(
    mut db: DB,
    #[allow(unused)]
    auth: AuthGuard<{ UserPermissionEnum::EXPENSES_WRITE as u32 }>,
    id: i32,
    expense: rocket::serde::json::Json<ExpensePatchRequest>,
) -> Result<Status, ApiError> {
    let req = expense.into_inner();

    let mut current_param_index = 1;

    let columns = vec![
        req.description.as_ref().map(|_| "description"),
        req.amount.as_ref().map(|_| "amount"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let sets = super::generate_sets_string(&columns, &mut current_param_index);

    if sets.is_empty() {
        return Ok(Status::NoContent);
    }

    let set_binds = vec![
        req.description.as_ref().map(|d| SqlType::String(d.clone())),
        req.amount.as_ref().map(|a| SqlType::BigDecimal(a.clone())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<SqlType>>();

    let query = format!(
        r#"
    UPDATE expenses
        SET {}
    WHERE id = ${}    
    "#,
        sets, current_param_index
    );

    let query = sqlx::query(&query);

    let query = set_binds
        .into_iter()
        .fold(query, |query, value| value.bind_to_query(query));

    query
        .bind(id)
        .execute(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::NotFound, format!("No expense with id {}", id))
            }
            _ => ApiError(Status::InternalServerError, e.to_string()),
        })?;

    Ok(Status::NoContent)
}

#[rocket::delete("/expenses/<id>")]
pub(super) async fn delete(
    mut db: DB,
    #[allow(unused)]
    auth: AuthGuard<{ UserPermissionEnum::EXPENSES_WRITE as u32 }>,
    id: i32,
) -> Result<Status, ApiError> {
    sqlx::query("DELETE FROM expenses WHERE id = $1")
        .bind(id)
        .execute(&mut **db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ApiError(Status::NotFound, format!("No expense with id {}", id))
            }
            _ => ApiError(Status::InternalServerError, e.to_string()),
        })?;

    Ok(Status::NoContent)
}
