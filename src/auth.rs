use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use std::ops::BitOr;

use crate::DB;

// TODO: Invalidating tokens
// How could we invalidate tokens?
// - Store tokens in a database

#[derive(Debug, Serialize, Deserialize)]
struct AuthInfo {
    /// Subject (whom the token refers to)
    sub: String,
    /// Expiration time (as UTC timestamp)
    exp: usize,
    /// User permissions
    permissions: UserPermissions,
}

// POST /auth/login
// {
//     "username": "user",
//     "password": "pass"
// }
// -> 200 OK
// {
//     "token": "..."
// }
// -> 401 Unauthorized
// {
//     "error": "Invalid username" | "Invalid password"
// }
#[derive(serde::Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

// TODO: Return error message
#[rocket::post("/auth/login", data = "<auth>")]
pub async fn login(auth: Json<LoginInfo>, cookies: &CookieJar<'_>) -> rocket::http::Status {
    let LoginInfo { username, password } = auth.into_inner();

    let conn = unsafe { DB.read().expect("Failed to aquire DB") };

    let mut stmt = conn
        .prepare("SELECT * FROM users WHERE username = ?")
        .unwrap();

    let mut rows = stmt.query(rusqlite::params![username]).unwrap();

    let mut user_exists = false;

    while let Some(row) = rows.next().unwrap_or_else(|e| {
        log::error!("Failed to get row: {}", e);
        None
    }) {
        user_exists = true;

        if row
            .get::<_, String>(2)
            .expect("Failed to get string at row 1")
            == password
        {
            let permissions: UserPermissions = row
                .get::<_, u32>(3)
                .expect("Failed to get permissions at row 3")
                .into();

            let mut cookie = Cookie::new(
                "auth_info",
                serde_json::json!(AuthInfo {
                    sub: username,
                    exp: time::OffsetDateTime::now_utc().unix_timestamp() as usize
                        + 60 * 60 * 24 * 7,
                    permissions,
                })
                .to_string(),
            );

            cookie.set_same_site(SameSite::None);
            cookie.set_http_only(true);
            
            // Only in release mode
            #[cfg(not(debug_assertions))]
            cookie.set_secure(true);

            cookies.add_private(cookie);

            return rocket::http::Status::Ok;
        } else {
            return rocket::http::Status::Forbidden;
        }
    }

    if !user_exists {
        return rocket::http::Status::Forbidden;
    }

    rocket::http::Status::InternalServerError
}

// POST /auth/logout
// -> 200 OK
// -> 401 Unauthorized
#[rocket::post("/auth/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> rocket::http::Status {
    if cookies.get_private("auth_info").is_some() {
        cookies.remove_private(Cookie::from("auth_info"));

        rocket::http::Status::Ok
    } else {
        rocket::http::Status::Unauthorized
    }
}

// GET /auth/status
// -> 200 OK
// {
//     "username": "user",
//     "permissions": 0
// }
// -> 401 Unauthorized
#[rocket::get("/auth/status")]
pub async fn status(cookies: &CookieJar<'_>) -> Result<Json<AuthInfo>, Status> {
    if let Some(cookie) = cookies.get_private("auth_info") {
        let auth_info: AuthInfo = serde_json::from_str(cookie.value()).unwrap();

        Ok(Json(auth_info))
    } else {
        Err(rocket::http::Status::Unauthorized)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct UserPermissions(u32);

impl UserPermissions {
    pub const PRODUCT_READ: UserPermissions = UserPermissions(0b0001);
    pub const PRODUCT_WRITE: UserPermissions = UserPermissions(0b0010);
    pub const ORDER_READ: UserPermissions = UserPermissions(0b0100);
    pub const ORDER_WRITE: UserPermissions = UserPermissions(0b1000);
    pub const USER_READ: UserPermissions = UserPermissions(0b0001_0000);
    pub const USER_WRITE: UserPermissions = UserPermissions(0b0010_0000);
    pub const MANAGE_DB: UserPermissions = UserPermissions(0b0100_0000);
    pub const ADMIN: UserPermissions = UserPermissions(0xFFFF_FFFF);
}

impl From<u32> for UserPermissions {
    fn from(permissions: u32) -> Self {
        Self(permissions)
    }
}

impl BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self.0 | rhs.0) }
    }
}

impl Into<u32> for UserPermissions {
    fn into(self) -> u32 {
        self.0
    }
}

impl UserPermissions {
    pub fn has_permission(&self, permission: &UserPermissions) -> bool {
        let permission = permission.0;

        (self.0 & permission) == permission
    }
}

pub fn verify_user_permissions(
    permissions: &UserPermissions,
    cookies: &CookieJar<'_>,
) -> Result<(), Status> {
    if let Some(cookie) = cookies.get_private("auth_info") {
        let auth_info: AuthInfo = serde_json::from_str(cookie.value()).unwrap();

        if auth_info.permissions.has_permission(permissions) {
            return Ok(());
        } else {
            return Err(Status::Forbidden);
        }
    }

    Err(Status::Unauthorized)
}
