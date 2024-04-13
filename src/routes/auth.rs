use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    serde::json::Json,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};

use crate::{db::DbConn, permissions::UserPermissions};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AuthInfo {
    /// Subject (whom the token refers to)
    sub: String,
    /// Expiry, none means session cookie
    expires_in: Option<i64>,
    /// User permissions
    permissions: UserPermissions,
}

// POST /auth/login
// {
//     "username": "user",
//     "password": "pass",
//     /// Optional, if not provided, the token will be a session cookie
//     "expires_in": 0
// }
// -> 200 OK
// -> 401 Unauthorized
#[derive(serde::Deserialize)]
pub(super) struct LoginData {
    username: String,
    password: String,
    #[serde(default)]
    expires_in: Option<i64>,
}

#[rocket::post("/auth/login", data = "<login_data>")]
pub(super) async fn login(
    mut conn: Connection<DbConn>,
    login_data: Json<LoginData>,
    cookies: &CookieJar<'_>,
) -> Result<Status, Status> {
    let LoginData {
        username,
        password,
        expires_in,
    } = login_data.into_inner();

    // NOTE: Assumes that usernames are unique
    let row = crate::db::sqlx::query(
        "SELECT username, password, salt, permissions FROM users WHERE username = $1 LIMIT 1",
    )
    .bind(&username)
    .fetch_one(&mut **conn)
    .await;

    if let Err(e) = row {
        match e {
            sqlx::Error::RowNotFound => return Ok(Status::Unauthorized),
            _ => {
                log::error!("DB error while fetching user: {:?}", e);
                return Err(Status::InternalServerError);
            }
        }
    }

    #[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
    pub struct UserRow {
        username: String,
        password: String,
        salt: String,
        permissions: i32,
    }

    let row = UserRow::from_row(&row.unwrap());

    if let Err(e) = row {
        log::error!("Error while parsing user row: {:?}", e);
        return Err(Status::InternalServerError);
    }

    let row = row.unwrap();

    let salted_hashed_password = salt_hash_password(&password, &row.salt);

    if row.password == salted_hashed_password {
        let permissions = UserPermissions::from(row.permissions as u32);

        let expiry_timestamp = expires_in.map(|expires_in| {
            rocket::time::OffsetDateTime::now_utc() + rocket::time::Duration::seconds(expires_in)
        });

        let mut cookie = Cookie::new(
            "auth_info",
            serde_json::json!(AuthInfo {
                sub: username,
                expires_in,
                permissions,
            })
            .to_string(),
        );

        // Set expiry if provided
        if let Some(expiry_timestamp) = expiry_timestamp {
            cookie.set_expires(expiry_timestamp);
        } else {
            cookie.set_expires(None);
        }

        cookie.set_same_site(SameSite::Lax);

        cookies.add_private(cookie);

        return Ok(Status::Ok);
    } else {
        return Ok(Status::Unauthorized);
    }
}

// POST /auth/logout
// -> 200 OK
// -> 401 Unauthorized
#[rocket::post("/auth/logout")]
pub(super) async fn logout(cookies: &CookieJar<'_>) -> Status {
    if cookies.get_private("auth_info").is_some() {
        cookies.remove_private(Cookie::from("auth_info"));

        Status::Ok
    } else {
        Status::Unauthorized
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
pub(super) async fn status(cookies: &CookieJar<'_>) -> Result<Json<AuthInfo>, Status> {
    if let Some(cookie) = cookies.get_private("auth_info") {
        let auth_info: AuthInfo = serde_json::from_str(cookie.value()).unwrap();

        // Refresh token
        let mut cookie = Cookie::new(
            "auth_info",
            serde_json::json!(AuthInfo {
                sub: auth_info.sub.clone(),
                expires_in: auth_info.expires_in,
                permissions: auth_info.permissions,
            })
            .to_string(),
        );

        // Set expiry if provided
        if let Some(expiry_timestamp) = auth_info.expires_in {
            let expiry_timestamp = rocket::time::OffsetDateTime::now_utc()
                + rocket::time::Duration::seconds(expiry_timestamp);
            cookie.set_expires(expiry_timestamp);
        } else {
            cookie.set_expires(None);
        }

        cookies.add_private(cookie);

        Ok(Json(auth_info))
    } else {
        Err(rocket::http::Status::Unauthorized)
    }
}

pub fn generate_salt() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    let salt: String = (0..32)
        .map(|_| rng.gen_range(0..16))
        .map(|n| format!("{:x}", n))
        .collect();

    salt
}

pub fn salt_hash_password(password: &str, salt: &str) -> String {
    use ring::digest::{digest, SHA256};

    let salted_password = format!("{}{}", password, salt);

    let hash = digest(&SHA256, salted_password.as_bytes());

    hash.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn create_user<'a, E>(
    username: &str,
    password: &str,
    permissions: UserPermissions,
    conn: E,
) -> Result<(), sqlx::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let salt = generate_salt();

    let salted_hashed_password = salt_hash_password(&password, &salt);

    crate::db::sqlx::query(
        "INSERT INTO users (username, password, salt, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(&username)
    .bind(&salted_hashed_password)
    .bind(&salt)
    .bind(permissions as i32)
    .execute(conn)
    .await?;

    Ok(())
}
