pub mod auth;
pub mod search;
pub mod inventory;

use bigdecimal::BigDecimal;
use rocket::{response::{self, Responder}, routes, Request};
use serde::{Deserialize, Serialize};
use sqlx::{database::HasArguments, encode::IsNull, postgres::PgArguments, query::{Query, QueryAs}, Encode, Postgres};

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
    ]
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
    fn bind_to_query_as<'a, T>(&'a self, query: QueryAs<'a, Postgres, T, PgArguments>) -> QueryAs<'a, Postgres, T, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }

    fn bind_to_query<'a>(&'a self, query: Query<'a, Postgres, PgArguments>) -> Query<'a, Postgres, PgArguments> {
        match self {
            SqlType::Int(i) => query.bind(i),
            SqlType::String(s) => query.bind(s),
            SqlType::BigDecimal(b) => query.bind(b),
            SqlType::Float(f) => query.bind(f),
            // SqlType::DateTime(d) => query.bind(d),
        }
    }
}
