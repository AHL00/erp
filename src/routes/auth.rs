use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    outcome::{try_outcome, Outcome},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use ts_rs::TS;

use crate::{
    db::DB,
    types::permissions::{UserPermission, UserPermissionVec},
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub(super) struct AuthInfo {
    /// Subject (whom the token refers to)
    username: String,
    /// User permissions
    permissions: UserPermissionVec,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AuthCookieInfo {
    username: String,
    expiry_time: Option<i64>,
    permissions: UserPermission,
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
    // No need to refresh cookie, so no AuthGuard needed

    let LoginData {
        username,
        password,
        expires_in,
    } = login_data.into_inner();

    #[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
    struct UserRow {
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
        let permissions = UserPermission::from(row.permissions as u32);

        let expiry_timestamp = expires_in.map(|expires_in| {
            rocket::time::OffsetDateTime::now_utc() + rocket::time::Duration::seconds(expires_in)
        });

        let mut cookie = Cookie::new(
            "auth_info",
            serde_json::json!(AuthCookieInfo {
                username: username,
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
        // TODO: Change to secure when HTTPS is enabled and using same domain for API and frontend
        cookie.set_secure(false);

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
    // No need to refresh cookie, so no AuthGuard needed
    if cookies.get_private("auth_info").is_some() {
        log::info!("Removing token cookie");

        cookies.remove_private(Cookie::from("auth_info"));

        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[derive(serde::Deserialize)]
pub(super) struct CreateUserData {
    username: String,
    password: String,
    permissions: UserPermissionVec,
}

// POST /auth/create_user [Permissions: ADMIN]
// {
//     "username": "user",
//     "password": "pass",
//     "permissions": ["ORDER_WRITE", "PRODUCT_READ"]
// }
// -> 200 OK
// -> 400 Bad Request
// -> 500 Internal Server Error
// TODO: Verify password, check existing user, make sure theres at least one admin, handle token expiry
#[rocket::post("/auth/create_user", data = "<create_user_data>")]
pub(super) async fn create_user(
    mut db: DB,
    _auth: AuthGuard<{ UserPermission::ADMIN as u32 }>,
    create_user_data: Json<CreateUserData>,
) -> Result<Status, Status> {
    let CreateUserData {
        username,
        password,
        permissions,
    } = create_user_data.into_inner();

    let result = add_user_to_db(&username, &password, permissions.flatten(), &mut **db).await;

    match result {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            log::error!("Failed to add user to DB: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[derive(serde::Serialize, TS)]
#[ts(export)]
pub(super) struct ListUserData {
    username: String,
    permissions: UserPermissionVec,
}

// GET /auth/list_users [Permissions: ADMIN]
// -> 200 OK
//     [
//         {
//             "username": "user",
//             "permissions": ["ORDER_WRITE", "PRODUCT_READ"]
//         }
//     ]
// -> 500 Internal Server Error
#[rocket::get("/auth/list_users")]
pub(super) async fn list_users(
    mut db: DB,
    _auth: AuthGuard<{ UserPermission::ADMIN as u32 }>,
) -> Result<Json<Vec<ListUserData>>, Status> {
    #[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
    pub struct ListUserRow {
        username: String,
        permissions: i32,
    }

    let rows: Result<Vec<ListUserRow>, sqlx::Error> =
        sqlx::query_as("SELECT username, permissions FROM users")
            .fetch_all(&mut **db)
            .await;

    match rows {
        Ok(rows) => {
            let users: Vec<ListUserData> = rows
                .iter()
                .map(|row| ListUserData {
                    username: row.username.clone(),
                    permissions: UserPermissionVec::split_from(UserPermission::from(
                        row.permissions as u32,
                    )),
                })
                .collect();

            Ok(Json(users))
        }
        Err(e) => {
            log::error!("Failed to list users: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

// GET /auth/permissions [Permissions: ADMIN]
// -> 200 OK
// -> 500 Internal Server Error
#[rocket::get("/auth/permissions")]
pub(super) async fn permissions(
    _auth: AuthGuard<{ UserPermission::ADMIN as u32 }>,
) -> Result<Json<Vec<UserPermission>>, Status> {
    Ok(Json(UserPermission::variants().to_vec()))
}

// DELETE /auth/delete_user [Permissions: ADMIN]
// {
//     "username": "user"
// }
// -> 200 OK
// -> 400 Bad Request
// -> 500 Internal Server Error
#[derive(serde::Deserialize)]
pub(super) struct DeleteUserData {
    username: String,
}

#[rocket::delete("/auth/delete_user", data = "<delete_user_data>")]
pub(super) async fn delete_user(
    mut db: DB,
    _auth: AuthGuard<{ UserPermission::ADMIN as u32 }>,
    delete_user_data: Json<DeleteUserData>,
) -> Result<Status, Status> {
    let DeleteUserData { username } = delete_user_data.into_inner();

    let result = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&mut **db)
        .await;

    match result {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            log::error!("Failed to delete user: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

// GET /auth/status
// -> 200 OK
// {
//     "username": "user",
//     "permissions": 0
// }
// -> 401 Unauthorized (if not logged in)
// TODO: Verify that user still exists? that will slow down the request though. Maybe have a table
// that stores records of changes such as deleted user or changed permissions. If this username
// is in that table, do whatever is needed.
// Every time any user is edited, they will be added to the refresh token table. This should also be
// in the auth guard. Would it be easier to save in temp file? Maybe, but that would be less secure.
// An easier way is to instead of returning old_info, retrieve the user from the DB again. This is the same
// number of DB calls.
#[rocket::get("/auth/status")]
pub(super) async fn status(
    // Will refresh the token cookie
    _auth: AuthGuard<0>,
    cookies: &CookieJar<'_>,
) -> Result<Json<AuthInfo>, Status> {
    if let Some(cookie) = cookies.get_private("auth_info") {
        let cookie_info: AuthCookieInfo = serde_json::from_str(cookie.value()).unwrap();

        Ok(Json(AuthInfo {
            username: cookie_info.username,
            permissions: cookie_info.permissions.split_into_vec(),
        }))
    } else {
        // Cookie not found
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

pub async fn add_user_to_db<'a, E>(
    username: &str,
    password: &str,
    permissions: UserPermission,
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

/// Mandatory guard that refreshes the token cookie on every request and checks permissions.
/// USE ON EVERY ROUTE TO KEEP PERMISSIONS UP TO DATE.
pub(super) struct AuthGuard<const PERMISSIONS: u32> {
    auth_info: AuthCookieInfo,
}

impl<const PERMISSIONS: u32> AuthGuard<PERMISSIONS> {
    pub fn get_auth_info(&self) -> &AuthCookieInfo {
        &self.auth_info
    }
}

/// Guard that checks if the user has the required permissions.
/// Pass in the required permissions as a const generic after converting them to a u32.
/// Input 0 for no permissions required.
#[rocket::async_trait]
impl<'r, const PERMISSIONS: u32> rocket::request::FromRequest<'r> for AuthGuard<PERMISSIONS> {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let cookies = request.cookies();

        if let Some(cookie) = cookies.get_private("auth_info") {
            let auth_info: AuthCookieInfo = serde_json::from_str(cookie.value()).expect("Failed to parse auth_info cookie");

            let mut db = try_outcome!(request
                .guard::<DB>()
                .await
                .map_error(|_| (Status::InternalServerError, ())));

            #[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
            struct UserRow {
                username: String,
                permissions: i32,
            }

            let user_row: Result<UserRow, sqlx::Error> = sqlx::query_as(
                "SELECT username, permissions FROM users WHERE username = $1 LIMIT 1",
            )
            .bind(&auth_info.username)
            .fetch_one(&mut **db)
            .await;

            if let Err(e) = user_row {
                match e {
                    sqlx::Error::RowNotFound => {
                        // User not found (maybe deleted), remove cookie.
                        log::info!("Username not found, removing token cookie");
                        cookies.remove_private(Cookie::from("auth_info"));
                        return Outcome::Error((Status::Unauthorized, ()));
                    }
                    _ => {
                        log::error!("DB error while fetching user: {:?}", e);
                        return Outcome::Error((Status::InternalServerError, ()));
                    }
                }
            }

            let user_row = user_row.unwrap();

            // User exists, new permissions received.
            let mut refresh_cookie = Cookie::new(
                "auth_info",
                serde_json::to_string(&AuthCookieInfo {
                    username: auth_info.username.clone(),
                    expiry_time: auth_info.expiry_time,
                    permissions: UserPermission::from(user_row.permissions as u32),
                })
                .unwrap(),
            );

            // Configure cookie
            refresh_cookie.set_expires(
                auth_info
                    .expiry_time
                    .map(|t| rocket::time::OffsetDateTime::from_unix_timestamp(t).unwrap()),
            );

            refresh_cookie.set_same_site(SameSite::Lax);
            // TODO: Change to secure when HTTPS is enabled and using same domain for API and frontend
            refresh_cookie.set_secure(false);

            log::info!("Refreshing token cookie");

            cookies.remove_private(Cookie::from("auth_info"));
            cookies.add_private(refresh_cookie);

            // Check for permissions
            if UserPermission::from(user_row.permissions as u32)
                .has_permission(&UserPermission::from(PERMISSIONS))
            {
                return Outcome::Success(AuthGuard { auth_info });
            } else {
                return Outcome::Error((Status::Forbidden, ()));
            }
        } else {
            return Outcome::Error((Status::Unauthorized, ()));
        }
    }
}