use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{uri::Origin, Cookie, CookieJar, SameSite, Status},
    serde::json::Json,
    Data,
};
use serde::{Deserialize, Serialize};
use std::{cell::LazyCell, ops::BitOr, sync::LazyLock};

use crate::{env, DB};

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

    let conn = unsafe { DB.write().await };

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

        if row.get::<_, String>(2).expect("Failed to get string at row 1") == password {
            let permissions: UserPermissions = row.get::<_, u32>(3).expect("Failed to get permissions at row 3").into();

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

            cookie.set_same_site(SameSite::Lax);

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
#[repr(u32)]
pub enum UserPermissions {
    ProductRead = 0b00000001,
    ProductWrite = 0b00000010,
    OrderRead = 0b00000100,
    OrderWrite = 0b00001000,
    UserRead = 0b00010000,
    UserWrite = 0b00100000,
    Admin = 0xFFFFFFFF,
}

impl BitOr for UserPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u32 | rhs as u32) }
    }
}

impl From<u32> for UserPermissions {
    fn from(permissions: u32) -> Self {
        match permissions {
            0b00000001 => UserPermissions::ProductRead,
            0b00000010 => UserPermissions::ProductWrite,
            0b00000100 => UserPermissions::OrderRead,
            0b00001000 => UserPermissions::OrderWrite,
            0b00010000 => UserPermissions::UserRead,
            0b00100000 => UserPermissions::UserWrite,
            0xFFFFFFFF => UserPermissions::Admin,
            _ => panic!("Invalid u32 -> permissions conversion"),
        }
    }
}

impl Into<u32> for UserPermissions {
    fn into(self) -> u32 {
        self as u32
    }
}

impl UserPermissions {
    pub fn has_permission(&self, permission: &UserPermissions) -> bool {
        let permission = *permission as u32;

        (*self as u32 & permission) == permission
    }
}
