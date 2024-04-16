pub mod auth;
pub mod inventory;
pub mod orders;
pub mod customers;
pub mod search;

use bigdecimal::BigDecimal;
use rocket::{
    http::Status,
    response::{self, Responder},
    routes,
    Request,
    serde::json
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgArguments,
    query::{Query, QueryAs},
    Postgres,
};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        auth::login,
        auth::status,
        auth::logout,
        auth::create_user,
        auth::delete_user,
        auth::list_users,
        auth::permissions,
        search::product,
        inventory::count,
        inventory::list,
        inventory::get,
        inventory::put,
        inventory::patch,
        inventory::post,
        orders::get,
    ]
}

// TODO: Overhaul all routes to use this error type
struct ApiError(pub Status, pub String);

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
    // DateTime(sqlx::types::chrono::NaiveDateTime),
}

impl SqlType {
    fn bind_to_query_as<'a, T>(
        &'a self,
        query: QueryAs<'a, Postgres, T, PgArguments>,
    ) -> QueryAs<'a, Postgres, T, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }

    fn bind_to_query<'a>(
        &'a self,
        query: Query<'a, Postgres, PgArguments>,
    ) -> Query<'a, Postgres, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }
}
