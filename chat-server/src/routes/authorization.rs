use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

use crate::errors::rocket_server_errors::server_error;
use crate::repositories::user::UserRepository;
use crate::utils::db_connection::{CacheConn, DbConn};
use bcrypt::verify;
use rand::{distr::Alphanumeric, Rng};
use rocket::{options, post, routes};

#[derive(serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[post{"/login", format="json", data="<credentials>"}]
pub async fn login(
    mut db: Connection<DbConn>,
    mut cache: Connection<CacheConn>,
    credentials: Json<Credentials>,
) -> Result<Value, Custom<Value>> {
    // Find the user by username
    let user = UserRepository::find_by_username(&mut db, &credentials.username)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                Custom(Status::Unauthorized, json!("Wrong credentials"))
            }
            _ => server_error(e.into()),
        })?;

    // Verify the password
    if verify(&credentials.password, &user.password_hash)
        .map_err(|_| Custom(Status::Unauthorized, json!("Wrong credentials")))?
    {
        // Generate a token
        let token = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect::<String>();

        cache
            .set_ex::<String, i32, ()>(format!("sessions/{}", token), user.id, 3 * 60 * 60)
            .await
            .map_err(|e| server_error(e.into()))?;

        // Return the token
        Ok(json!({ "token": token }))
    } else {
        // Password verification failed
        Err(Custom(Status::Unauthorized, json!("Wrong credentials")))
    }
}

#[options("/<_..>")]
pub fn options() -> &'static str {
    ""
}

pub fn routes() -> Vec<rocket::Route> {
    routes![login, options]
}
