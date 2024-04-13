use crate::{
    db::DB,
    types::{permissions::UserPermissions, product::Product},
};

use rocket::{
    http::{CookieJar, Status},
    serde::json::Json,
};

use super::auth::verify_user_permissions;

#[rocket::get("/search/product?<query>&<count>&<distance>")]
pub async fn product(
    mut db: DB,
    query: Option<String>,
    count: Option<usize>,
    distance: Option<f32>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Product>>, Status> {
    // If anything is none, return malformed request
    let query = match query {
        Some(query) => query,
        None => return Err(Status::BadRequest),
    };

    let mut count = match count {
        Some(count) => count,
        None => return Err(Status::BadRequest),
    };

    let distance = match distance {
        Some(distance) => distance,
        None => return Err(Status::BadRequest),
    };

    // Check if user has permissions and is logged in
    verify_user_permissions(&UserPermissions::PRODUCT_READ, cookies)?;

    let products: Vec<Product> = sqlx::query_as("SELECT * FROM products")
        .fetch_all(&mut **db)
        .await
        .map_err(|e| {
            log::error!("Failed to get products: {}", e);
            Status::InternalServerError
        })?;
    
    let products: Vec<Product> = products
        .iter()
        .filter(|product| {
            let sorenson_dice = strsim::sorensen_dice(&product.name, &query);
            let similarity = ((sorenson_dice * 2.0)/*+ (lcsseq_ratio * 2.0)*/).clamp(0.0, 1.0);

            if similarity >= distance as f64 {
                count -= 1;

                if count == 0 {
                    return false;
                }

                return true;
            }

            false
        })
        .map(|product| product.clone())
        .collect();

    Ok(Json::from(products))
}
