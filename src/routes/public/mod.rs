pub mod inventory;

// Pass on utility functions to submodules as super::_;
pub use super::*;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        inventory::count,
        inventory::get,
        inventory::list,
        inventory::search,
    ]
}
