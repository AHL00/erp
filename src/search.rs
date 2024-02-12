use crate::{auth::{verify_user_permissions, UserPermissions}, Product, DB};
use rocket::{http::{CookieJar, Status}, serde::json::Json};

#[rocket::get("/search/product?<query>&<count>&<distance>")]
pub async fn product(query: Option<String>, count: Option<usize>, distance: Option<f32>, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Status> {
    // If anything is none, return malformed request
    let query = match query {
        Some(query) => query,
        None => return Err(Status::BadRequest),
    };

    let count = match count {
        Some(count) => count,
        None => return Err(Status::BadRequest),
    };

    let distance = match distance {
        Some(distance) => distance,
        None => return Err(Status::BadRequest),
    };

    // Check if user has permissions and is logged in
    verify_user_permissions(&UserPermissions::PRODUCT_READ, cookies)?;

    let db = unsafe {DB.read().expect("Failed to get DB")};

    let mut stmt = db.prepare("SELECT * FROM products").expect("Failed to prepare SQL");

    let mut rows = stmt.query([]).expect("Failed to query");

    let mut results = vec![];

    while let Some(row) = rows.next().unwrap_or_else(|e| {
        log::error!("Failed to get row: {}", e);
        None
    }) {
        if results.len() > count {
            break;
        }

        let name: String = row.get(1).unwrap();        
        let sorenson_dice = strsim::sorensen_dice(&name, &query);
        let similarity = ((sorenson_dice * 2.0) /*+ (lcsseq_ratio * 2.0)*/).clamp(0.0, 1.0);

        if similarity >= distance as f64 {
            let product = Product {
                id: row.get(0).unwrap(),
                name,
                price: row.get(3).unwrap(),
                stock: row.get(4).unwrap(),
                quantity_per_box: row.get(4).unwrap()
            };

            results.push(product);
        }   
    }

    Ok(Json::from(results))
}