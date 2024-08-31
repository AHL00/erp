use std::collections::HashMap;

use bigdecimal::BigDecimal;
use rocket::{http::Status, time::Date};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    db::DB,
    routes::{
        auth::AuthGuard,
        orders::{get_order_total, Order, OrderItem, OrderItemRow, OrderMetaRow},
    },
    types::permissions::UserPermissionEnum,
};

use super::{
    orders::{total, OrderMeta, OrderTotal},
    ApiError,
};

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS, Hash, Eq, PartialEq)]
#[ts(export)]
pub(super) enum ReportRequestType {
    Revenue,
    Profit,
    Expenses,
    Product,
    Receivable,
    Payable,
}

impl ReportRequestType {
    pub fn variants() -> Vec<Self> {
        vec![
            Self::Revenue,
            Self::Profit,
            Self::Expenses,
            Self::Product,
            Self::Receivable,
            Self::Payable,
        ]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) enum ReportFilter {
    UserId(i32),
    ProductId(i32),
    CustomerId(i32),
    SupplierId(i32),
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) struct ReportRequest {
    start_date: chrono::DateTime<chrono::Utc>,
    end_date: chrono::DateTime<chrono::Utc>,
    filters: Vec<ReportFilter>,
    report_types: Vec<ReportRequestType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ts_rs::TS)]
#[ts(export)]
pub(super) struct Report {
    start_date: chrono::DateTime<chrono::Utc>,
    end_date: chrono::DateTime<chrono::Utc>,
    orders: Vec<Order>,
    expenses: Vec<((), ())>,
    purchases: Vec<((), ())>,
    data: HashMap<ReportRequestType, BigDecimal>,
}

#[rocket::post("/reports/create", data = "<report_request>")]
#[allow(private_interfaces)]
pub async fn create_report(
    mut db: DB,
    report_request: rocket::serde::json::Json<ReportRequest>,
    _auth: AuthGuard<{ UserPermissionEnum::REPORTS as u32 }>,
) -> Result<rocket::serde::json::Json<Report>, ApiError> {
    let ReportRequest {
        start_date,
        end_date,
        filters,
        report_types,
    } = report_request.into_inner();

    let mut res = Report {
        start_date: start_date,
        end_date: end_date,
        orders: vec![],
        expenses: vec![],
        purchases: vec![],
        data: HashMap::new(),
    };

    let start_date = start_date.naive_utc();
    let end_date = end_date.naive_utc();

    let filters_sql_order_meta = filters
        .iter()
        .map(|f| match f {
            ReportFilter::UserId(id) => format!("AND orders.created_by_user_id = {}", id),
            ReportFilter::CustomerId(id) => format!("AND orders.customer_id = {}", id),
            _ => "".to_string(),
        })
        .collect::<Vec<String>>()
        .join(" ");

    let filters_sql_order_items = filters
        .iter()
        .map(|f| match f {
            ReportFilter::ProductId(id) => format!("AND order_items.product_id = {}", id),
            _ => "".to_string(),
        })
        .collect::<Vec<String>>()
        .join(" ");

    #[derive(Debug, FromRow)]
    struct OrderMetaRowWithItems {
        #[sqlx(flatten)]
        order_meta: OrderMetaRow,
        items: sqlx_core::types::Json<Vec<OrderItemRow>>,
    }

    let orders_query_string = format!(
        r#"
    WITH order_meta AS (
        SELECT 
            orders.id,
            orders.date_time,
            orders.amount_paid,
            orders.retail,
            orders.notes,
            row_to_json(customers) AS customer,
            row_to_json(users) AS created_by_user
        FROM orders
            INNER JOIN customers ON orders.customer_id = customers.id
            INNER JOIN users ON orders.created_by_user_id = users.id
        WHERE date_time BETWEEN $1 AND $2
        {}
    )
    SELECT
        order_meta.*,
        (
            SELECT json_agg(
                json_build_object(
                    'id', order_items.id,
                    'inventory', row_to_json(inventory),
                    'price', order_items.price,
                    'quantity', order_items.quantity
                )
            )
            FROM order_items
                INNER JOIN inventory ON order_items.inventory_id = inventory.id
            WHERE order_items.order_id = order_meta.id
            {}
        ) AS items
    FROM order_meta
    "#,
        filters_sql_order_meta, filters_sql_order_items
    );

    // Get every order between the start and end date
    let orders: Vec<(OrderMeta, Vec<OrderItem>)> = sqlx::query_as(orders_query_string.as_str())
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&mut **db)
        .await
        .map_err(|e| ApiError(Status::InternalServerError, e.to_string()))
        .map(|rows: Vec<OrderMetaRowWithItems>| {
            rows.into_iter()
                .map(|row| {
                    (
                        OrderMeta::from(row.order_meta),
                        row.items
                            .0
                            .into_iter()
                            .map(|item| OrderItem::from(item))
                            .collect(),
                    )
                })
                .collect()
        })?;

    // TODO
    let expenses = ();

    // TODO
    let purchases = ();

    // Calculate revenue
    let mut total_revenue = BigDecimal::from(0);
    let mut total_receivable = BigDecimal::from(0);
    for (order_meta, order_items) in orders {
        let order_total: BigDecimal = order_items
            .iter()
            .map(|item| &item.price * BigDecimal::from(item.quantity))
            .sum();

        let receivable = (&order_total - &order_meta.amount_paid).max(BigDecimal::from(0));
        total_revenue += order_total;
        total_receivable += receivable;

        let order = Order {
            meta: order_meta,
            items: order_items,
        };

        res.orders.push(order);
    }

    // Calculate profit

    // Fill return data
    for ty in ReportRequestType::variants() {
        if report_types.contains(&ty) {
            let value = match ty {
                ReportRequestType::Revenue => total_revenue.clone(),
                ReportRequestType::Profit => BigDecimal::from(0),
                ReportRequestType::Expenses => BigDecimal::from(0),
                ReportRequestType::Product => BigDecimal::from(0),
                ReportRequestType::Receivable => total_receivable.clone(),
                ReportRequestType::Payable => BigDecimal::from(0),
            };

            res.data.insert(ty, value);
        }
    }

    Ok(rocket::serde::json::Json::from(res))
}
