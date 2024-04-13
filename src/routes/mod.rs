pub mod auth;
pub mod search;

use rocket::routes;

pub fn routes() -> Vec<rocket::Route> {
    routes![auth::login, auth::status, auth::logout, search::product]
}
