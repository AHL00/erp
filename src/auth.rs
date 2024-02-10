use serde::{Deserialize, Serialize};

use crate::{UserPermissions, DB};

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
pub async fn login(mut req: tide::Request<()>) -> tide::Result {
    #[derive(serde::Deserialize)]
    struct Auth {
        username: String,
        password: String,
    }

    let Auth { username, password } = req.body_json().await?;

    let conn = unsafe { DB.write().await };

    let mut stmt = conn
        .prepare("SELECT * FROM users WHERE username = ?")
        .unwrap();

    let mut rows = stmt.query(rusqlite::params![username]).unwrap();

    let mut user_exists = false;

    while let Some(row) = rows.next().unwrap() {
        user_exists = true;

        if row.get::<_, String>(1).unwrap() == password {
            let permissions: u8 = row.get(3).unwrap();
            let permissions: UserPermissions = unsafe { std::mem::transmute(permissions) };

            let session_mut = req.session_mut();

            // TODO: Refresh expiration time automatically
            session_mut.insert("login_info", AuthInfo {
                sub: username,
                exp: time::OffsetDateTime::now_utc().unix_timestamp() as usize + 60 * 60 * 24 * 7,
                permissions,
            }).expect("Failed to insert login info into session");

            let response = tide::Response::new(200);

            return Ok(response);
        } else {
            let mut response = tide::Response::new(401);
            response.set_body(serde_json::json!({
                "error": "Invalid password"
            }));

            return Ok(response);
        }
    }

    if !user_exists {
        let mut response = tide::Response::new(401);
        response.set_body(serde_json::json!({
            "error": "Invalid username"
        }));

        return Ok(response);
    }

    Ok(tide::Response::new(500))
}

// POST /auth/logout
// -> 200 OK
// -> 401 Unauthorized
pub async fn logout(mut req: tide::Request<()>) -> tide::Result {
    let session_mut = req.session_mut();

    session_mut.remove("login_info");

    Ok(tide::Response::new(200))
}


// GET /auth/info
// -> 200 OK
// {
//     "username": "user",
//     "permissions": 0
// }
// -> 401 Unauthorized
// {
//     "error": "Not logged in"
// }
pub async fn info(req: tide::Request<()>) -> tide::Result {
    let session = req.session();

    let auth_info: Option<AuthInfo> = session.get("login_info");

    if let Some(login_info) = auth_info {
        let mut response = tide::Response::new(200);

        response.set_body(serde_json::json!({
            "username": login_info.sub,
            "permissions": login_info.permissions as u8
        }));

        return Ok(response);
    }

    let mut response = tide::Response::new(401);

    response.set_body(serde_json::json!({
        "error": "Not logged in"
    }));

    Ok(response)
}
