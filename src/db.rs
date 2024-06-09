use std::future::Future;

use pollster::FutureExt;
use rocket::{Ignite, Rocket, Sentinel};
use rocket_db_pools::Connection;
use sqlx::{PgConnection, Row};

pub use rocket_db_pools::sqlx;

use crate::{
    routes::{auth::add_user_to_db, ApiError},
    types::permissions::{UserPermissionEnum, UserPermissions},
};

pub type DB = Connection<DatabaseConnection>;

pub struct DatabaseConnection(sqlx::PgPool);

impl rocket_db_pools::Database for DatabaseConnection {
    const NAME: &'static str = "main_db";
    type Pool = sqlx::PgPool;
}

impl From<sqlx::PgPool> for DatabaseConnection {
    fn from(pool: sqlx::PgPool) -> Self {
        log::info!("Creating a new database connection pool");

        let schema_exists = sqlx::query("SELECT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'public')")
            .fetch_one(&pool)
            .block_on();

        if let Err(e) = schema_exists {
            log::error!("Error checking if schema exists: {:?}", e);
            std::process::exit(1);
        }

        let schema_exists: bool = schema_exists.unwrap().get(0);

        let schema_str = include_str!("schema.sql");

        if !schema_exists {
            log::warn!("Schema does not exist");
        }

        log::info!("Running schema creation in case a table is missing");

        let create_schema = sqlx::raw_sql(schema_str).execute(&pool).block_on();

        if let Err(e) = create_schema {
            log::error!("Error creating schema: {:?}", e);
            std::process::exit(1);
        }

        // If no users with admin rights exist, create a default admin user
        // This will ensure that the first user can always log in
        // TODO: In the future, when implementing user deletion, make sure that there is always at
        // least one admin user in order to not trigger this
        let users = sqlx::query("SELECT * FROM users")
            .fetch_all(&pool)
            .block_on();

        if let Err(e) = users {
            log::error!("Error fetching users: {:?}", e);
            std::process::exit(1);
        }

        let users = users.unwrap();

        if users.is_empty() {
            log::warn!(
                "No users found, creating default admin user (username: admin, password: admin)"
            );

            let create_user_res = add_user_to_db(
                "admin",
                "admin",
                UserPermissions(UserPermissionEnum::ADMIN as u32),
                &pool,
            )
            .block_on();

            if let Err(e) = create_user_res {
                log::error!("Error creating default admin user: {:?}", e);
                std::process::exit(1);
            }
        }

        Self(pool)
    }
}

impl std::ops::Deref for DatabaseConnection {
    type Target = sqlx::PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DatabaseConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for DatabaseConnection {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let pool = request.guard::<DatabaseConnection>().await.unwrap();
        rocket::request::Outcome::Success(pool)
    }
}

#[rocket::async_trait]
impl Sentinel for DatabaseConnection {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        if rocket.state::<DatabaseConnection>().is_none() {
            return true;
        }

        // TODO: What does this do?
        if !rocket
            .catchers()
            .any(|c| c.code == Some(400) && c.base == "/")
        {
            return true;
        }

        false
    }
}

/// Trait which allows structs to be retrieved from the database by id.
pub trait FromDB: Sized {
    fn from_db(id: i32, db: &mut crate::db::DB) -> impl Future<Output = Result<Self, ApiError>>;
}
