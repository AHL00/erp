use rocket::{http::{Cookie, CookieJar, SameSite, Status}, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::{cell::LazyCell, ops::BitOr, sync::LazyLock};
use tide::{Middleware, Next, Request};

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

    while let Some(row) = rows.next().unwrap() {
        user_exists = true;

        if row.get::<_, String>(1).unwrap() == password {
            let permissions: u32 = row.get(3).unwrap();
            let permissions: UserPermissions = unsafe { std::mem::transmute(permissions) };

            let mut cookie = Cookie::new("auth_info", serde_json::json!(
                AuthInfo {
                    sub: username,
                    exp: time::OffsetDateTime::now_utc().unix_timestamp() as usize
                        + 60 * 60 * 24 * 7,
                    permissions,
                }
            ).to_string());

            cookie.set_same_site(SameSite::Lax);

            cookies.add_private(cookie);

            return rocket::http::Status::Ok;
        } else {
            return rocket::http::Status::Unauthorized;
        }
    }

    if !user_exists {
        return rocket::http::Status::Unauthorized;
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

impl UserPermissions {
    pub fn has_permission(&self, permission: &UserPermissions) -> bool {
        let permission = *permission as u32;

        (*self as u32 & permission) == permission
    }
}

pub struct UserPermissionsMiddleware {
    permission: UserPermissions,
}

impl UserPermissionsMiddleware {
    pub fn new(permission: UserPermissions) -> Self {
        Self { permission }
    }
}

impl<State> Middleware<State> for UserPermissionsMiddleware
where
    State: Clone + Send + Sync + 'static,
{
    fn handle<'life0, 'life1, 'async_trait>(
        &'life0 self,
        request: Request<State>,
        next: Next<'life1, State>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = tide::Result> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let session = request.session();

            let auth_info: Option<AuthInfo> = session.get("login_info");

            if let Some(login_info) = auth_info {
                if login_info.permissions.has_permission(&self.permission) {
                    Ok(next.run(request).await)
                } else {
                    let mut response = tide::Response::new(403);

                    response.set_body(serde_json::json!({
                        "error": "Insufficient permissions"
                    }));

                    Ok(response)
                }
            } else {
                let mut response = tide::Response::new(401);

                response.set_body(serde_json::json!({
                    "error": "Not logged in"
                }));

                Ok(response)
            }
        })
    }
}
