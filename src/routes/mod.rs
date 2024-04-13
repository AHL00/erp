pub mod auth;

use rocket::routes;

pub fn routes() -> Vec<rocket::Route> {
    routes![auth::login, auth::status, auth::logout]
}
