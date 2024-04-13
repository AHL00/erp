pub mod auth;
pub mod search;

use rocket::routes;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        auth::login,
        auth::status,
        auth::logout,
        auth::create_user,
        auth::delete_user,
        auth::list_users,
        auth::permissions,
        search::product
    ]
}
