pub mod auth;
pub mod customers;
pub mod expenses;
pub mod inventory;
pub mod orders;
pub mod reports;
pub mod search;
pub mod backup;

pub mod public;

use bigdecimal::BigDecimal;
use chrono::Utc;
use inventory::InventoryItem;
use rocket::{
    http::Status,
    response::{self, Responder},
    routes,
    serde::json,
    Request,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgArguments,
    query::{Query, QueryAs},
    Postgres,
};

use crate::db::FromDB;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        auth::login,
        auth::status,
        auth::logout,
        auth::create_user,
        auth::delete_user,
        auth::list_users,
        auth::permissions,
        inventory::count,
        inventory::list,
        inventory::get,
        inventory::patch,
        inventory::post,
        inventory::search,
        orders::get,
        orders::get_items,
        orders::count,
        orders::list,
        orders::post,
        orders::update_items,
        orders::preview_update_items,
        orders::patch,
        orders::delete,
        orders::total,
        customers::get,
        customers::count,
        customers::list,
        customers::post,
        customers::patch,
        customers::search,
        reports::create_report,
        reports::create_expense_report,
        reports::create_order_report,
        expenses::get,
        expenses::count,
        expenses::list,
        expenses::post,
        expenses::patch,
        expenses::delete,
        backup::backup,
        // backup::restore,
        // customers::delete,
    ]
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
struct StockUpdate {
    pub id: i32,
    pub date_time: sqlx::types::chrono::DateTime<Utc>,
    pub inventory_id: i32,
    pub created_by_user_id: i32,
    pub delta: i32,
    pub order_item_id: Option<i32>,
    pub order_id: Option<i32>,
    pub purchase_item_id: Option<i32>,
    pub purchase_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
struct CreateStockUpdate {
    pub inventory: InventoryItem,
    pub created_by_user_id: i32,
    pub delta: i32,
    pub order_item_id: Option<i32>,
    pub order_id: Option<i32>,
    pub purchase_item_id: Option<i32>,
    pub purchase_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
struct ListRequest {
    range: ListRange,
    sorts: Vec<ListSort>,
    filters: Vec<ListFilter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct ListRange {
    /// Number of items to send
    count: i32,
    /// First item's index
    offset: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct ListSort {
    column: String,
    order: SortOrder,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
struct ListFilter {
    column: String,
    operator: FilterOperator,
    value: SqlType,
}

// TODO: Overhaul all routes to use this error type
pub struct ApiError(pub Status, pub String);

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct ErrorResponse {
            error: String,
        }

        let json = json::to_string(&ErrorResponse {
            error: self.1.clone(),
        })
        .unwrap();

        let json_bytes = json.into_boxed_str().into_boxed_bytes();

        response::Response::build()
            .status(self.0)
            .sized_body(json_bytes.len(), std::io::Cursor::new(json_bytes))
            .ok()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        // Handle all generic database errors here
        ApiError(Status::InternalServerError, error.to_string())
    }
}

struct ApiReturn<T: Serialize>(pub Status, pub T);

impl<'r, T: Serialize> Responder<'r, 'static> for ApiReturn<T> {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let json = json::to_string(&self.1).unwrap();

        let json_bytes = json.into_boxed_str().into_boxed_bytes();

        response::Response::build()
            .status(self.0)
            .sized_body(json_bytes.len(), std::io::Cursor::new(json_bytes))
            .ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "UPPERCASE")]
enum SortOrder {
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
enum FilterOperator {
    #[serde(rename = "=")]
    Eq,
    #[serde(rename = "!=")]
    Ne,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = ">=")]
    Ge,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = "LIKE")]
    Like,
}

impl FilterOperator {
    pub fn to_sql(&self) -> &'static str {
        match self {
            FilterOperator::Eq => "=",
            FilterOperator::Ne => "!=",
            FilterOperator::Gt => ">",
            FilterOperator::Lt => "<",
            FilterOperator::Ge => ">=",
            FilterOperator::Le => "<=",
            FilterOperator::Like => "LIKE",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
enum SqlType {
    Int(i32),
    String(String),
    BigDecimal(BigDecimal),
    Float(f64),
    Boolean(bool),
    Null,
    // DateTime(sqlx::types::chrono::NaiveDateTime),
}

impl SqlType {
    fn bind_to_query_as<'a, T>(
        self,
        query: QueryAs<'a, Postgres, T, PgArguments>,
    ) -> QueryAs<'a, Postgres, T, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            SqlType::Boolean(b) => query.bind(b),
            SqlType::Null => query.bind(None::<i32>),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }

    fn bind_to_query<'a>(
        self,
        query: Query<'a, Postgres, PgArguments>,
    ) -> Query<'a, Postgres, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            SqlType::Boolean(b) => query.bind(b),
            SqlType::Null => query.bind(None::<i32>),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }
}

fn generate_sets_string(columns: &[&'static str], current_param: &mut i32) -> String {
    let mut sets = String::new();
    for column in columns.iter() {
        sets.push_str(column);
        sets.push_str(" = $");
        sets.push_str(&(current_param).to_string());
        sets.push_str(", ");
        *current_param += 1;
    }
    sets.pop();
    sets.pop();
    sets
}

fn generate_sorts_string(sorts: &[ListSort]) -> String {
    let mut sorts_string = String::new();
    for (i, sort) in sorts.iter().enumerate() {
        if i == 0 {
            sorts_string.push_str("ORDER BY ");
        }

        sorts_string.push_str(&sort.column);
        sorts_string.push(' ');
        sorts_string.push_str(match &sort.order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        });
        sorts_string.push_str(", ");
    }
    sorts_string.pop();
    sorts_string.pop();
    sorts_string
}

fn generate_filters_string(
    filters: &[ListFilter],
    current_arg: &mut i32,
) -> (String, Vec<SqlType>) {
    let mut filter_binds = vec![];

    let filter_string = filters
        .iter()
        .enumerate()
        .map(|(i, filter)| {
            let str = format!(
                "{} {} {} ${}",
                if i == 0 { "WHERE" } else { "AND" },
                filter.column,
                filter.operator.to_sql(),
                current_arg
            );

            filter_binds.push(filter.value.clone());

            *current_arg += 1;

            str
        })
        .collect::<Vec<_>>()
        .join(" ");

    (filter_string, filter_binds)
}
