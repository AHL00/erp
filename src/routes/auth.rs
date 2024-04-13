use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

use crate::{db::DB, types::permissions::UserPermissions};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AuthInfo {
    /// Subject (whom the token refers to)
    sub: String,
    /// Expiry timestamp, none means session cookie
    expiry_time: Option<i64>,
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
    mut db: DB,
    login_data: Json<LoginData>,
    cookies: &CookieJar<'_>,
) -> Result<Status, Status> {
    let LoginData {
        username,
        password,
        expires_in,
    } = login_data.into_inner();

    #[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
    pub struct UserRow {
        username: String,
        password: String,
        salt: String,
        permissions: i32,
    }

    // NOTE: Assumes that usernames are unique
    let row: Result<UserRow, sqlx::Error> = sqlx::query_as(
        "SELECT username, password, salt, permissions FROM users WHERE username = $1 LIMIT 1",
    )
    .bind(&username)
    .fetch_one(&mut **db)
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
                expiry_time: expiry_timestamp.map(|t| t.unix_timestamp()),
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
        cookie.set_secure(true);

        log::info!("Sending token cookie");

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
        log::info!("Removing token cookie");

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
        let old_info: AuthInfo = serde_json::from_str(cookie.value()).unwrap();

        // Refresh token
        let refresh_info = AuthInfo {
            sub: old_info.sub.clone(),
            expiry_time: old_info.expiry_time,
            permissions: old_info.permissions,
        };

        // Update expiry time if provided
        if let Some(old_expiry_time) = old_info.expiry_time {
            let mut refresh_cookie =
                Cookie::new("auth_info", serde_json::to_string(&refresh_info).unwrap());
            refresh_cookie.set_expires(
                rocket::time::OffsetDateTime::from_unix_timestamp(old_expiry_time)
                    .expect("Failed to parse expiry time"),
            );
            refresh_cookie.set_same_site(SameSite::Lax);
            refresh_cookie.set_secure(true);

            log::info!("Refreshing token cookie");

            cookies.add_private(refresh_cookie);
        } else {
            let mut refresh_cookie =
                Cookie::new("auth_info", serde_json::to_string(&refresh_info).unwrap());
            
            refresh_cookie.set_expires(None);
            refresh_cookie.set_same_site(SameSite::Lax);
            refresh_cookie.set_secure(true);

            log::info!("Refreshing token cookie, session cookie");

            cookies.add_private(refresh_cookie);
        }

        Ok(Json(old_info))
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
    .bind(permissions.into_u32() as i32)
    .execute(conn)
    .await?;

    Ok(())
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
